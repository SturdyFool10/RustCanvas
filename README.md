<div align="center">

# 🎨 RustCanvas

### *Replacing expensive multiplayer canvas and design programs with a free and open source alternative*

[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

*A high-performance, real-time collaborative canvas application built with Rust and modern web technologies*

[Features](#features) •
[Quick Start](#quick-start) •
[Installation](#installation) •
[Usage](#usage) •
[Contributing](#contributing)

</div>

---

## ✨ Features

- 🚀 **Blazing Fast Performance** - Built with Rust for maximum speed and efficiency
- 🌐 **Real-time Collaboration** - Multiple users can draw and design together simultaneously
- 💰 **Completely Free** - No subscriptions, no hidden costs, forever free and open source
- 🎯 **Professional Tools** - Advanced drawing tools and design capabilities
- 🔒 **Secure** - Built with security and privacy in mind
- 📱 **Cross-Platform** - Works on Windows, macOS, and Linux
- 🎨 **Modern UI** - Clean, intuitive interface designed for productivity
- ⚡ **WebSocket Communication** - Ultra-low latency real-time updates
- 🗃️ **Persistent Storage** - SQLite database for reliable data persistence

## 🚀 Quick Start

Get RustCanvas up and running in just a few commands:

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (for protocol buffer generation)
- [Protocol Buffers Compiler (protoc)](https://protobuf.dev/downloads/) - Must be accessible via `protoc` command in terminal
- [Git](https://git-scm.com/)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/RustCanvas.git
   cd RustCanvas
   ```

2. **Install Node.js dependencies**
   ```bash
   npm install
   ```
   This installs `protoc-gen-js` which is required for generating JavaScript protocol buffer files.

   **Note:** Make sure `protoc` is installed and available in your PATH. You can verify this by running:
   ```bash
   protoc --version
   ```
   If this command fails, please install Protocol Buffers from the [official downloads page](https://protobuf.dev/downloads/).

3. **Build the protocol crate**
   ```bash
   cargo build -p protocol
   ```
   This runs the build script that generates both Rust and JavaScript code from the protocol buffer definitions.

4. **Build the entire project**
   ```bash
   cargo build --release
   ```

5. **Locate and run the executable**

   The compiled executable will be located at:
   ```bash
   # On Windows
   target/release/rustcanvas.exe

   # On Linux/macOS
   target/release/rustcanvas
   ```

   Run it directly:
   ```bash
   # Windows
   ./target/release/rustcanvas.exe

   # Linux/macOS
   ./target/release/rustcanvas
   ```

That's it! 🎉 RustCanvas should now be built and found in the `target/release` directory.

## 🏗️ Architecture

RustCanvas is built as a modular workspace with the following crates:

```
crates/
├── rustcanvas/      # Main application entry point
├── webserver/       # HTTP server and WebSocket handling
├── protocol/        # Protocol buffer definitions and networking
├── db/              # Database layer and persistence
├── appstate/        # Application state management
├── config/          # Configuration management
├── authentication/ # User authentication and authorization
├── utils/           # Shared utilities
├── macros/          # Custom derive macros
└── prettylogs/      # Enhanced logging functionality
```

## 🛠️ Development

### Building from Source

**Prerequisites for Development:**
Make sure you have `protoc` installed and accessible in your PATH:

```bash
    protoc --version
```

If `protoc` is not installed, follow these platform-specific instructions:

**Windows:**
1. Download the latest release from [Protocol Buffers releases](https://github.com/protocolbuffers/protobuf/releases)
2. Extract the `protoc.exe` to a directory (e.g., `C:\protoc\bin\`)
3. Add that directory to your PATH environment variable
4. Restart your terminal and verify with `protoc --version`

**macOS:**
```bash
    # Using Homebrew
    brew install protobuf

    # Using MacPorts
    sudo port install protobuf3-cpp
```

**Linux (Ubuntu/Debian):**
```bash
    # Using apt
    sudo apt update
    sudo apt install protobuf-compiler

    # Using snap
    sudo snap install protobuf --classic
```

**Linux (CentOS/RHEL/Fedora):**
```bash
    # Using dnf/yum
    sudo dnf install protobuf-compiler
    # or
    sudo yum install protobuf-compiler
```

**Building the Project:**

```bash
    # First, ensure npm dependencies are installed
    npm install

    # Build protocol crate (generates proto bindings)
    cargo build -p protocol

    # Development build (faster compilation)
    cargo build

    # Optimized release build (maximum performance)
    cargo build --release
```

### Running the Application

```bash
    # Run development version
    ./target/debug/rustcanvas      # Linux/macOS
    ./target/debug/rustcanvas.exe  # Windows

    # Run optimized version
    ./target/release/rustcanvas      # Linux/macOS
    ./target/release/rustcanvas.exe  # Windows
```

### Protocol Buffer Development

The protocol crate includes a build script that automatically generates both Rust and JavaScript code from protocol buffer definitions. If you modify the protocol buffer definitions in `crates/protocol/proto/messages.proto`, you'll need to rebuild:

```bash
    # Rebuild protocol crate (regenerates bindings)
    cargo build -p protocol

    # Then rebuild the entire project
    cargo build
```

The build script will:
- Generate Rust code using `prost`
- Generate JavaScript code using `protoc-gen-js`
- Copy generated files to appropriate locations

### Performance Profiles

RustCanvas includes highly optimized build profiles:

- **Development**: Fast compilation with some optimizations for dependencies
- **Release**: Maximum performance with LTO, panic=abort, and aggressive optimizations

## 📊 Performance

RustCanvas is designed for maximum performance:

- **Ultra-low latency** WebSocket communication
- **Memory efficient** with Rust's zero-cost abstractions
- **CPU optimized** with advanced compiler optimizations
- **Scalable** architecture supporting thousands of concurrent users

## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

1. 🍴 Fork the repository
2. 🔧 Create a feature branch (`git checkout -b feature/amazing-feature`)
3. 💡 Make your changes
4. ✅ Add tests if applicable
5. 📝 Commit your changes (`git commit -m 'Add amazing feature'`)
6. 🚀 Push to the branch (`git push origin feature/amazing-feature`)
7. 🎯 Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass before submitting PR

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Rust community for providing excellent tools and libraries
- Contributors who help make RustCanvas better
- Users who provide feedback and bug reports

## 🐛 Issues & Support

Found a bug or have a feature request? Please open an issue on our [GitHub Issues](https://github.com/yourusername/RustCanvas/issues) page.

## 🌟 Star History

If you find RustCanvas useful, please consider giving it a star! ⭐

---

<div align="center">

**Made with ❤️ and Rust**

*Empowering creativity through open source collaboration*

</div>
