mod core;
mod vector;
mod ai;
mod lua;
mod file;
mod query;
mod cli;
mod server;
mod auth;

use clap::{Parser, Subcommand};
use crate::core::{RocksDBWrapper, NamespaceManager};
use crate::vector::UsearchWrapper;
use crate::ai::{LLMWrapper, EmbeddingWrapper};
use crate::lua::LuaVM;
use crate::file::FileStorage;
use crate::query::executor::QueryExecutor;
use crate::auth::AuthManager;
use anyhow::Result;
use candle_core::Device;
use crate::server::api::run_server;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, default_value = "cpu")]
    device: String,

    #[arg(long)]
    model_path: String,

    #[arg(long)]
    tokenizer_path: String,
}

#[derive(Subcommand)]
enum Commands {
    Cli,
    Server { port: Option<u16> },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let device = match cli.device.as_str() {
        "cpu" => Device::Cpu,
        "cuda" => Device::new_cuda(0)?,
        _ => anyhow::bail!("Invalid device specified"),
    };

    let namespace_manager = NamespaceManager::new();
    let llm = LLMWrapper::new(cli.model_path.into(), cli.tokenizer_path.into(), device)?;
    let embedding = EmbeddingWrapper::new(fastembed::EmbeddingModel::AllMiniLML6V2)?;
    let lua_vm = LuaVM::new(std::path::PathBuf::from("path/to/luarocks"))?;
    let file_storage = FileStorage::new("path/to/file/storage")?;
    let mut auth_manager = AuthManager::new();

    // Add a default admin user
    auth_manager.add_user("admin", vec![
        "select".to_string(),
        "insert".to_string(),
        "update".to_string(),
        "delete".to_string(),
        "create_namespace".to_string(),
        "delete_namespace".to_string(),
        "upload_file".to_string(),
        "process_file".to_string(),
        "generate_embedding".to_string(),
        "similarity_search".to_string(),
        "llm_query".to_string(),
    ]);

    let query_executor = QueryExecutor::new(
        namespace_manager,
        llm,
        embedding,
        lua_vm,
        file_storage,
        auth_manager,
        5,  // max_concurrent_llm
        10, // max_concurrent_embedding
    );

    match &cli.command {
        Some(Commands::Cli) => {
            cli::console::run(query_executor)?;
        }
        Some(Commands::Server { port }) => {
            let port = port.unwrap_or(3000);
            run_server(port, query_executor).await?;
        }
        None => {
            println!("Please specify a command: cli or server");
        }
    }

    Ok(())
}