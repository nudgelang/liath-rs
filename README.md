# 🚀 Liath

[![Rust](https://img.shields.io/badge/Rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Experimental](https://img.shields.io/badge/status-experimental-orange.svg)](https://github.com/nudgelang/liath)

Liath is an experimental high-performance version of [nudgelang/liath](https://github.com/nudgelang/liath). It is an AI-first database system that integrates AI capabilities directly into its core functionality. It combines traditional database operations with AI features such as language model inference and vector similarity search, all accessible through a Lua-based query language.

## ✨ Key Features

- 🔌 **RocksDB Storage**: High-performance key-value operations with multi-threaded column families
- 🤖 **AI Integration**: 
  - Integrated Language Model (LLM) inference using Candle
  - Vector embeddings with FastEmbed
  - Similarity search powered by USearch
- 📝 **Lua Query Language**: Flexible and powerful data manipulation through rlua
- 📁 **File Operations**: Built-in file storage and processing
- 🔐 **Authentication**: Secure user management system
- 🌐 **Server & CLI**: 
  - HTTP API with Axum
  - Command-line interface
- ⚡ **Performance**: 
  - Async runtime with Tokio
  - Multi-threaded operations
  - GPU acceleration support (CUDA)

## 🛠️ Prerequisites

- Rust (latest stable version)
- CUDA toolkit (optional, for GPU acceleration)

## 🚀 Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/nudgelang/liath-rs.git
   cd liath-rs
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## 💻 Usage

### CLI Mode

```bash
cargo run --release -- --device cpu --model-path /path/to/model.gguf --tokenizer-path /path/to/tokenizer.json cli
```

### Server Mode

```bash
cargo run --release -- --device cuda --model-path /path/to/model.gguf --tokenizer-path /path/to/tokenizer.json server
```

## 📝 Lua Query Examples

```lua
-- Create a namespace
create_namespace("users")

-- Insert data
insert("users", "user123", "Alice")

-- Generate text using the LLM
local response = llm_query("What is the capital of France?", 100)
print(response)

-- Perform a similarity search
local embedding = generate_embedding("Hello, world!")
local results = similarity_search("users", embedding, 5)
print(results)
```

## ⚙️ Configuration

Create a `config.toml` file to customize your setup:

```toml
[database]
data_dir = "/path/to/data"

[llm]
max_concurrent = 5

[embedding]
max_concurrent = 10

[auth]
default_user = "admin"
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.