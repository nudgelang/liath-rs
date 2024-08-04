# Whitematter

Whitematter is an AI-first database system that integrates AI capabilities directly into its core functionality. It combines traditional database operations with AI features such as language model inference and vector similarity search, all accessible through a Lua-based query language.

## Features

- RocksDB-based storage for high-performance key-value operations
- Integrated Language Model (LLM) for text generation and AI-assisted queries
- Vector database functionality for similarity search
- Lua-based query language for flexible and powerful data manipulation
- File storage and processing capabilities
- Authentication and authorization system
- Support for transactions and advanced RocksDB features
- CLI and Server modes for versatile usage

## Prerequisites

- Rust (latest stable version)
- CUDA toolkit (optional, for GPU acceleration)

## Installation

1. Clone the repository:

2. Build the project:
   ```
   cargo build --release
   ```

## Usage

To run the database, you need to provide the paths to a GGUF model file and a tokenizer file. You can download these from the Hugging Face Model Hub.

### CLI Mode

```
cargo run --release -- --device cpu --model-path /path/to/model.gguf --tokenizer-path /path/to/tokenizer.json cli
```

### Server Mode

```
cargo run --release -- --device cuda --model-path /path/to/model.gguf --tokenizer-path /path/to/tokenizer.json server
```

Replace `cpu` with `cuda` to use GPU acceleration (if available).

## Lua Query Examples

Here are some example queries you can run in the CLI mode:

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

## Configuration

You can configure various aspects of the database by modifying the `config.toml` file (create one if it doesn't exist):

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.