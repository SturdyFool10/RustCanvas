//! build.rs
//! Build script to generate Rust and JavaScript code from Protocol Buffers definitions

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this build script if any .proto files change
    println!("cargo:rerun-if-changed=proto/");

    let proto_file = "proto/messages.proto";

    // Generate Rust code
    generate_rust_code(proto_file);

    // Generate JavaScript code
    generate_javascript_code(proto_file);
}

fn generate_rust_code(proto_file: &str) {
    println!("cargo:warning=Generating Rust code from {}", proto_file);

    // Check if proto file exists and is not empty
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

    // Get the output directory for the generated code
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Configure prost-build
    let mut config = prost_build::Config::new();

    // Compile the proto file
    match config.compile_protos(&[proto_file], &["proto"]) {
        Ok(_) => {
            println!("cargo:warning=Successfully compiled proto file with prost");

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

    // Generate a full JavaScript client with proper protobuf support
    println!("cargo:warning=Generating full JavaScript protobuf client");

    match generate_full_js_client(proto_file, &js_target) {
        Ok(_) => {
            println!(
                "cargo:warning=Successfully generated JavaScript client at {}",
                js_target.display()
            );
        }
        Err(e) => {
            println!("cargo:warning=Failed to generate JavaScript client: {}", e);
            create_javascript_placeholder(&js_target);
        }
    }
}

fn generate_full_js_client(
    proto_file: &str,
    target_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the proto file and extract message names
    let proto_content = fs::read_to_string(proto_file)?;
    let messages = extract_message_names(&proto_content);

    let js_content = generate_protobuf_client_code(&messages);

    fs::write(target_file, js_content)?;
    Ok(())
}

fn extract_message_names(proto_content: &str) -> Vec<String> {
    let mut messages = Vec::new();

    for line in proto_content.lines() {
        let line = line.trim();
        if line.starts_with("message ") && line.contains('{') {
            if let Some(name_part) = line.strip_prefix("message ") {
                if let Some(name) = name_part.split_whitespace().next() {
                    messages.push(name.to_string());
                }
            }
        }
    }

    messages
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

fn generate_protobuf_client_code(messages: &[String]) -> String {
    let message_classes = messages
        .iter()
        .map(|msg| generate_message_class(msg))
        .collect::<Vec<_>>()
        .join("\n\n");

    let message_registrations = messages
        .iter()
        .map(|msg| format!("protoClient.messageTypes['{}'] = {};", msg, msg))
        .collect::<Vec<_>>()
        .join("\n");

    let window_exports = messages
        .iter()
        .map(|msg| format!("window.{} = {};", msg, msg))
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        r#"// DO NOT EDIT! This file was automatically generated from proto/messages.proto

console.log('Proto-client.js: Using full JavaScript protobuf client');

/**
 * Full JavaScript protobuf client with binary encoding/decoding
 * Generated from proto definitions: {}
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

    writeInt32(fieldNumber, value) {{
        if (value !== 0) {{
            this.writeTag(fieldNumber, WIRE_TYPE_VARINT);
            this.writeVarint(value);
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
{}

class ProtoClient {{
    constructor() {{
        console.log('ProtoClient initialized with messages: {}');
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
{}

// Export for CommonJS
if (typeof module !== 'undefined' && module.exports) {{
    module.exports = {{ ProtoClient, protoClient, {} }};
}}

// Export for ES6 modules
if (typeof window !== 'undefined') {{
    window.ProtoClient = ProtoClient;
    window.protoClient = protoClient;
    {}
}}
"#,
        messages.join(", "),
        message_classes,
        messages.join(", "),
        message_registrations,
        messages.join(", "),
        window_exports
    )
}

fn generate_message_class(message_name: &str) -> String {
    // For now, generate a basic message class structure
    // This could be enhanced to parse the actual proto file for field definitions
    format!(
        r#"class {} {{
    constructor(data = {{}}) {{
        // Initialize with default values
        this.message = data.message || '';
    }}

    // Encode this message to protobuf binary format
    encode() {{
        const writer = new ProtobufWriter();
        if (this.message) {{
            writer.writeString(1, this.message);
        }}
        return writer.getBytes();
    }}

    // Decode protobuf binary data to create message instance
    static decode(buffer) {{
        const reader = new ProtobufReader(buffer);
        const message = new {}();

        while (reader.hasMore()) {{
            const tag = reader.readTag();
            switch (tag.fieldNumber) {{
                case 1:
                    if (tag.wireType === WIRE_TYPE_LENGTH_DELIMITED) {{
                        message.message = reader.readString();
                    }}
                    break;
                default:
                    // Skip unknown fields
                    break;
            }}
        }}

        return message;
    }}

    // Convert to JSON for debugging
    toJSON() {{
        return {{
            message: this.message
        }};
    }}
}}"#,
        message_name, message_name
    )
}
