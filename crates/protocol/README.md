# Protocol Crate

This crate handles Protocol Buffer (protobuf) message definitions and code generation for both Rust backend and JavaScript frontend communication.

## How It Works

The build system (`build.rs`) automatically generates code from your `.proto` files during compilation:

### Generated Files

1. **Rust Code**: `src/generated/mod.rs`
   - Contains Rust structs and enums generated from your proto definitions
   - Uses the `prost` crate for serialization/deserialization
   - Automatically included in the library

2. **JavaScript Code**: `../webserver/src/htmlsrc/proto-client.js`
   - Contains JavaScript classes for client-side protobuf handling
   - Generated using `protoc` when available
   - Falls back to placeholder implementation if `protoc` is not installed

### Build Process

When you run `cargo build`, the build script:

1. **Checks** if `proto/messages.proto` exists and is not empty
2. **Generates Rust code** using `prost-build`
3. **Copies** generated Rust code to `src/generated/mod.rs`
4. **Attempts JavaScript generation** using `protoc`
5. **Creates placeholders** if generation fails

## Setup

### For Rust (Always Works)

The Rust code generation uses `prost` and works out of the box. No additional setup required.

### For JavaScript (Requires protoc)

To get proper JavaScript protobuf support, install the Protocol Compiler:

#### Windows
```powershell
# Using Chocolatey
choco install protoc

# Or download from: https://github.com/protocolbuffers/protobuf/releases
```

#### macOS
```bash
brew install protobuf
```

#### Linux
```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# CentOS/RHEL
sudo yum install protobuf-compiler
```

### Verify Installation
```bash
protoc --version
```

## Usage

### In Rust Code

```rust
use protocol::{helpers, YourMessageType};
use bytes::Bytes;

// Encode a message
let message = YourMessageType { /* fields */ };
let bytes: Bytes = helpers::encode_message(&message);

// Decode a message
let decoded: YourMessageType = helpers::decode_message(&bytes)?;

// Generate message IDs
let id = helpers::new_message_id();
```

### In JavaScript Code

```javascript
// Import the generated client
// <script src="proto-client.js"></script>

const client = new ProtoClient();

// Check if real protobuf is available
if (client.isPlaceholder()) {
    console.log("Using JSON fallback - install protoc for protobuf support");
}

// Encode/decode (works with both real protobuf and JSON fallback)
const encoded = client.encode(message);
const decoded = client.decode(encoded);
```

## File Structure

```
crates/protocol/
├── build.rs              # Build script for code generation
├── proto/
│   └── messages.proto     # Your protobuf definitions (add your messages here)
├── src/
│   ├── lib.rs            # Main library file
│   ├── generated/
│   │   └── mod.rs        # Generated Rust code (auto-generated)
│   └── helpers.rs        # Helper functions for encoding/decoding
└── js_out/               # Temporary JS generation directory
```

## Adding Your Protocol Definitions

1. Edit `proto/messages.proto` with your message definitions:

```protobuf
syntax = "proto3";

message YourMessage {
  uint64 id = 1;
  string content = 2;
  repeated string tags = 3;
}
```

2. Run `cargo build -p protocol` to regenerate code

3. Use the generated types in your Rust and JavaScript code

## Troubleshooting

### "Proto file is empty" Warning
- Add your message definitions to `proto/messages.proto`
- The system creates placeholder code when the proto file is empty

### "protoc command failed" Warning
- Install the Protocol Compiler (`protoc`)
- The system falls back to JSON-based placeholder client

### Generated Files Not Found
- Check that the build completed successfully
- Verify file permissions in the generated directories
- Run `cargo clean -p protocol && cargo build -p protocol` to force regeneration

## Development

The build script automatically handles:
- ✅ Empty or missing proto files
- ✅ Missing `protoc` installation
- ✅ File permission issues
- ✅ Cross-platform compatibility
- ✅ Incremental builds (only rebuilds when proto files change)

## Dependencies

- **Rust**: `prost`, `prost-build`, `bytes`
- **JavaScript**: `protoc` (optional, falls back to JSON)