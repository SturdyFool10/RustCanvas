//! build.rs
//! Build script to generate Rust and JavaScript code from Protocol Buffers definitions

use prost_reflect::prost::Message as ProstMessage;
use std::env;
use std::fs;
use std::path::Path; // Needed for FileDescriptorSet::decode

fn main() {
    // Tell Cargo to rerun this build script if any .proto files change
    println!("cargo:rerun-if-changed=proto/");

    // Whitelist of proto files for runtime (ONLY these will be included in Rust/JS/descriptor set)
    const RUNTIME_PROTOS: &[&str] = &["proto/messages.proto"];

    // Generate Rust code and descriptor set for runtime protos only
    generate_rust_code_and_descriptor(RUNTIME_PROTOS);

    // Generate JavaScript code using the descriptor set
    let descriptor_path = std::path::Path::new("src/descriptor_set.bin");
    if let Ok(descriptor_bytes) = std::fs::read(descriptor_path) {
        let js_code = generate_protobuf_client_code(&descriptor_bytes);
        let js_target = std::path::Path::new("../webserver/src/htmlsrc/proto-client.js");
        if let Err(e) = std::fs::write(js_target, js_code) {
            println!("cargo:warning=Failed to write JS client: {}", e);
        } else {
            println!(
                "cargo:warning=Successfully generated JavaScript client at {:?}",
                js_target
            );
        }
    } else {
        println!("cargo:warning=Could not read descriptor set for JS codegen");
    }
}

fn generate_rust_code_and_descriptor(proto_files: &[&str]) {
    println!(
        "cargo:warning=Generating Rust code and descriptor set from {:?}",
        proto_files
    );

    // Check if all proto files exist and are not empty
    for proto_file in proto_files {
        match fs::metadata(proto_file) {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    println!(
                        "cargo:warning=Proto file {} is empty, creating placeholder",
                        proto_file
                    );
                    create_rust_placeholder();
                    return;
                }
            }
            Err(e) => {
                println!(
                    "cargo:warning=Proto file {} not accessible: {}",
                    proto_file, e
                );
                create_rust_placeholder();
                return;
            }
        }
    }

    // Get the output directory for the generated code
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Configure prost-build
    let mut config = prost_build::Config::new();

    // Set the descriptor set output path
    let descriptor_set_path = out_path.join("descriptor_set.pb");
    config.file_descriptor_set_path(&descriptor_set_path);

    // Compile all proto files
    match config.compile_protos(proto_files, &["proto"]) {
        Ok(_) => {
            println!("cargo:warning=Successfully compiled proto files with prost");

            // Find the generated file and copy it to our source tree
            let mut found_generated = false;
            if let Ok(entries) = fs::read_dir(&out_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "rs") {
                        copy_rust_generated_code(&path);
                        found_generated = true;
                        break;
                    }
                }
            }

            if !found_generated {
                println!("cargo:warning=No generated Rust file found, creating placeholder");
                create_rust_placeholder();
            }

            // Copy the descriptor set to a known location in the crate for runtime use
            let crate_descriptor_path = Path::new("src/descriptor_set.bin");
            if let Err(e) = fs::copy(&descriptor_set_path, crate_descriptor_path) {
                println!(
                    "cargo:warning=Failed to copy descriptor set to src/descriptor_set.bin: {}",
                    e
                );
            } else {
                println!("cargo:warning=Copied descriptor set to src/descriptor_set.bin");
            }
        }
        Err(e) => {
            println!("cargo:warning=Failed to compile proto file: {}", e);
            create_rust_placeholder();
        }
    }
}

fn copy_rust_generated_code(generated_file: &Path) {
    let target_dir = Path::new("src/generated");
    let target_file = target_dir.join("mod.rs");

    // Create the directory if it doesn't exist
    if !target_dir.exists() {
        if let Err(e) = fs::create_dir_all(target_dir) {
            println!("cargo:warning=Failed to create generated directory: {}", e);
            return;
        }
    }

    // Read the generated content
    match fs::read_to_string(generated_file) {
        Ok(content) => {
            // Add a header comment
            let final_content = format!(
                "// DO NOT EDIT! This file was automatically generated from proto/messages.proto\n\n{}",
                content
            );

            // Write to the target location
            if let Err(e) = fs::write(&target_file, final_content) {
                println!(
                    "cargo:warning=Failed to write generated Rust code to {}: {}",
                    target_file.display(),
                    e
                );
            } else {
                println!(
                    "cargo:warning=Successfully generated Rust code at {}",
                    target_file.display()
                );
            }
        }
        Err(e) => {
            println!("cargo:warning=Failed to read generated Rust code: {}", e);
            create_rust_placeholder();
        }
    }
}

fn create_rust_placeholder() {
    let target_dir = Path::new("src/generated");
    let target_file = target_dir.join("mod.rs");

    // Create the directory if it doesn't exist
    if !target_dir.exists() {
        if let Err(e) = fs::create_dir_all(target_dir) {
            println!("cargo:warning=Failed to create generated directory: {}", e);
            return;
        }
    }

    let placeholder = r#"// DO NOT EDIT! This file was automatically generated from proto/messages.proto
// This is a placeholder because proto compilation failed

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlaceholderMessage {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
"#;

    if let Err(e) = fs::write(&target_file, placeholder) {
        println!("cargo:warning=Failed to write placeholder Rust code: {}", e);
    } else {
        println!("cargo:warning=Created placeholder Rust code");
    }
}

fn generate_javascript_code(proto_file: &str) {
    println!(
        "cargo:warning=Generating JavaScript code from {}",
        proto_file
    );

    // Debug: Print current working directory
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    println!(
        "cargo:warning=Current working directory: {}",
        current_dir.display()
    );

    // Debug: Check if proto file exists from current directory
    let proto_path = current_dir.join(proto_file);
    println!(
        "cargo:warning=Looking for proto file at: {}",
        proto_path.display()
    );
    println!("cargo:warning=Proto file exists: {}", proto_path.exists());

    // Get the workspace root
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:warning=CARGO_MANIFEST_DIR: {}", manifest_dir);
    let workspace_root = Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."));
    println!("cargo:warning=Workspace root: {}", workspace_root.display());

    // Target file for JavaScript output
    let js_target = workspace_root.join("crates/webserver/src/htmlsrc/proto-client.js");
    println!("cargo:warning=JS target: {}", js_target.display());

    // Create parent directory if it doesn't exist
    if let Some(parent) = js_target.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                println!("cargo:warning=Failed to create JS target directory: {}", e);
                create_javascript_placeholder(&js_target);
                return;
            }
        }
    }

    // Check if proto file exists and is not empty
    match fs::metadata(proto_file) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                println!(
                    "cargo:warning=Proto file {} is empty, creating JS placeholder",
                    proto_file
                );
                create_javascript_placeholder(&js_target);
                return;
            }
        }
        Err(e) => {
            println!(
                "cargo:warning=Proto file {} not accessible: {}",
                proto_file, e
            );
            create_javascript_placeholder(&js_target);
            return;
        }
    }

    // Try to use protoc to generate JavaScript
    let temp_dir = env::temp_dir().join("rustcanvas_proto_js");
    println!("cargo:warning=Using temp directory: {}", temp_dir.display());

    if let Err(e) = fs::create_dir_all(&temp_dir) {
        println!("cargo:warning=Failed to create temp directory: {}", e);
        create_javascript_placeholder(&js_target);
        return;
    }
}

fn create_javascript_placeholder(target_file: &Path) {
    let placeholder = r#"// DO NOT EDIT! This file was automatically generated from proto/messages.proto
// This is a placeholder because protoc was not available, failed, or proto file was empty

console.warn('Proto-client.js: Using placeholder implementation');

// Placeholder JavaScript protobuf implementation
class ProtoClient {
    constructor() {
        console.warn('Using placeholder proto client - add your proto definitions to enable real protobuf support');
    }

    // Placeholder methods - replace with actual proto-generated methods
    encode(message) {
        console.warn('Placeholder encode called - no real protobuf encoding available');
        return JSON.stringify(message);
    }

    decode(data) {
        console.warn('Placeholder decode called - no real protobuf decoding available');
        try {
            return JSON.parse(data);
        } catch (e) {
            console.error('Failed to parse JSON fallback:', e);
            return {};
        }
    }

    // Helper method to check if real protobuf is available
    isPlaceholder() {
        return true;
    }
}

// Export for CommonJS
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ProtoClient };
}

// Export for ES6 modules
if (typeof window !== 'undefined') {
    window.ProtoClient = ProtoClient;
}
"#;

    if let Err(e) = fs::write(target_file, placeholder) {
        println!(
            "cargo:warning=Failed to write JavaScript placeholder: {}",
            e
        );
    } else {
        println!(
            "cargo:warning=Created JavaScript placeholder at {}",
            target_file.display()
        );
    }
}

fn generate_protobuf_client_code(descriptor_bytes: &[u8]) -> String {
    use prost_reflect::prost::Message as ProstMessage;
    use prost_reflect::{DescriptorPool, FieldDescriptor, Kind, Value};

    let file_descriptor_set =
        ProstMessage::decode(descriptor_bytes).expect("Failed to decode descriptor set");
    let pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set)
        .expect("Failed to build DescriptorPool");

    let mut message_classes = Vec::new();
    let mut message_registrations = Vec::new();
    let mut window_exports = Vec::new();
    let mut message_names = Vec::new();

    for message in pool.all_messages() {
        let mut field_info = Vec::new();
        for field in message.fields() {
            let field_name = field.name().to_string();
            let field_number = field.number();
            let (js_type, wire_type) = match field.kind() {
                Kind::String => ("string".to_string(), "string".to_string()),
                Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Enum(_) => {
                    ("number".to_string(), "int32".to_string())
                }
                Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => {
                    ("number".to_string(), "int64".to_string())
                }
                Kind::Uint32 | Kind::Fixed32 => ("number".to_string(), "uint32".to_string()),
                Kind::Uint64 | Kind::Fixed64 => ("number".to_string(), "uint64".to_string()),
                Kind::Bool => ("boolean".to_string(), "bool".to_string()),
                // For now, treat bytes and messages as strings (could be improved)
                Kind::Bytes => ("string".to_string(), "string".to_string()),
                Kind::Message(_) => ("object".to_string(), "string".to_string()),
                _ => ("string".to_string(), "string".to_string()),
            };
            field_info.push((field_name, field_number, js_type, wire_type));
        }
        message_classes.push(generate_message_class(&message, &field_info));
        message_registrations.push(format!(
            "protoClient.messageTypes['{}'] = {};",
            message.name(),
            message.name()
        ));
        window_exports.push(format!("window.{} = {};", message.name(), message.name()));
        message_names.push(message.name().to_string());
    }

    format!(
        r#"// DO NOT EDIT! This file was automatically generated from proto/messages.proto

console.log('Proto-client.js: Using full JavaScript protobuf client');

/**
 * Full JavaScript protobuf client with binary encoding/decoding
 * Generated from proto definitions: {message_names}
 */

// Protobuf wire types
const WIRE_TYPE_VARINT = 0;
const WIRE_TYPE_FIXED64 = 1;
const WIRE_TYPE_LENGTH_DELIMITED = 2;
const WIRE_TYPE_START_GROUP = 3;
const WIRE_TYPE_END_GROUP = 4;
const WIRE_TYPE_FIXED32 = 5;

// Utility functions for protobuf encoding/decoding
class ProtobufWriter {{
    constructor() {{
        this.buffer = [];
    }}

    writeVarint(value) {{
        while (value >= 0x80) {{
            this.buffer.push((value & 0xFF) | 0x80);
            value >>>= 7;
        }}
        this.buffer.push(value & 0xFF);
    }}

    writeTag(fieldNumber, wireType) {{
        this.writeVarint((fieldNumber << 3) | wireType);
    }}

    writeString(fieldNumber, value) {{
        if (value && value.length > 0) {{
            const utf8Bytes = new TextEncoder().encode(value);
            this.writeTag(fieldNumber, WIRE_TYPE_LENGTH_DELIMITED);
            this.writeVarint(utf8Bytes.length);
            this.buffer.push(...utf8Bytes);
        }}
    }}

    getBytes() {{
        return new Uint8Array(this.buffer);
    }}
}}

class ProtobufReader {{
    constructor(buffer) {{
        this.buffer = new Uint8Array(buffer);
        this.pos = 0;
    }}

    readVarint() {{
        let result = 0;
        let shift = 0;
        while (this.pos < this.buffer.length) {{
            const byte = this.buffer[this.pos++];
            result |= (byte & 0x7F) << shift;
            if ((byte & 0x80) === 0) {{
                return result;
            }}
            shift += 7;
        }}
        throw new Error('Invalid varint');
    }}

    readTag() {{
        const tag = this.readVarint();
        return {{
            fieldNumber: tag >>> 3,
            wireType: tag & 0x7
        }};
    }}

    readString() {{
        const length = this.readVarint();
        const bytes = this.buffer.slice(this.pos, this.pos + length);
        this.pos += length;
        return new TextDecoder().decode(bytes);
    }}

    hasMore() {{
        return this.pos < this.buffer.length;
    }}
}}

// Generated message classes
{message_classes}

class ProtoClient {{
    constructor() {{
        console.log('ProtoClient initialized with messages: {message_names}');
        this.messageTypes = {{}};
    }}

    // Encode message to binary protobuf format
    encode(message) {{
        if (!message || typeof message.encode !== 'function') {{
            throw new Error('Message must have an encode method');
        }}
        return message.encode();
    }}

    // Decode binary data based on message type
    decode(messageClass, data) {{
        if (!messageClass || typeof messageClass.decode !== 'function') {{
            throw new Error('Message class must have a decode method');
        }}
        return messageClass.decode(data);
    }}

    // Create a new message instance
    create(messageType, data = {{}}) {{
        const MessageClass = this.messageTypes[messageType];
        if (!MessageClass) {{
            throw new Error(`Unknown message type: ${{messageType}}`);
        }}
        return new MessageClass(data);
    }}

    // Check if real protobuf is available
    isPlaceholder() {{
        return false; // This is a full protobuf implementation
    }}

    // Get list of available message types
    getMessageTypes() {{
        return Object.keys(this.messageTypes);
    }}
}}

// Register message types
const protoClient = new ProtoClient();
{message_registrations}

// Export for CommonJS
if (typeof module !== 'undefined' && module.exports) {{
    module.exports = {{ ProtoClient, protoClient, {message_names} }};
}}

// Export for ES6 modules
if (typeof window !== 'undefined') {{
    window.ProtoClient = ProtoClient;
    window.protoClient = protoClient;
    {window_exports}
}}
"#,
        message_names = message_names.join(", "),
        message_classes = message_classes.join("\n\n"),
        message_registrations = message_registrations.join("\n"),
        window_exports = window_exports.join("\n    "),
    )
}

fn generate_message_class(
    message: &prost_reflect::MessageDescriptor,
    field_info: &[(String, u32, String, String)],
) -> String {
    // field_info: Vec of (field_name, field_number, js_type, wire_type)
    let class_name = message.name();
    let mut ctor_lines = String::new();
    let mut encode_lines = String::new();
    let mut decode_cases = String::new();
    let mut tojson_lines = String::new();

    for (field_name, field_number, js_type, wire_type) in field_info {
        ctor_lines.push_str(&format!(
            "        this.{0} = data.{0} !== undefined ? data.{0} : {1};\n",
            field_name,
            match js_type.as_str() {
                "string" => "''",
                "number" => "0",
                "boolean" => "false",
                _ => "null",
            }
        ));
        // Encode
        encode_lines.push_str(&format!(
            "        if (this.{0} !== {1}) {{ writer.{2}({3}, this.{0}); }}\n",
            field_name,
            match js_type.as_str() {
                "string" => "''",
                "number" => "0",
                "boolean" => "false",
                _ => "null",
            },
            match wire_type.as_str() {
                "string" => "writeString",
                "int32" | "int64" | "uint32" | "uint64" | "bool" => "writeVarint",
                _ => "writeString", // fallback
            },
            field_number
        ));
        // Decode
        decode_cases.push_str(&format!(
            "                case {0}:\n                    if (tag.wireType === {1}) {{ message.{2} = reader.{3}(); }}\n                    break;\n",
            field_number,
            match wire_type.as_str() {
                "string" => "WIRE_TYPE_LENGTH_DELIMITED",
                "int32" | "int64" | "uint32" | "uint64" | "bool" => "WIRE_TYPE_VARINT",
                _ => "WIRE_TYPE_LENGTH_DELIMITED",
            },
            field_name,
            match wire_type.as_str() {
                "string" => "readString",
                "int32" | "int64" | "uint32" | "uint64" | "bool" => "readVarint",
                _ => "readString",
            }
        ));
        // toJSON
        tojson_lines.push_str(&format!("            {0}: this.{0},\n", field_name));
    }

    format!(
        r#"class {class_name} {{
    constructor(data = {{}}) {{
{ctor_lines}    }}

    // Encode this message to protobuf binary format
    encode() {{
        const writer = new ProtobufWriter();
{encode_lines}        return writer.getBytes();
    }}

    // Decode protobuf binary data to create message instance
    static decode(buffer) {{
        const reader = new ProtobufReader(buffer);
        const message = new {class_name}();

        while (reader.hasMore()) {{
            const tag = reader.readTag();
            switch (tag.fieldNumber) {{
{decode_cases}                default:
                    // Skip unknown fields
                    break;
            }}
        }}

        return message;
    }}

    // Convert to JSON for debugging
    toJSON() {{
        return {{
{tojson_lines}        }};
    }}
}}
"#
    )
}
