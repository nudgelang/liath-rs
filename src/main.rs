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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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

    let namespace_manager = NamespaceManager::new();
    let llm = LLMWrapper::new("path/to/your/llm/model")?;
    let embedding = EmbeddingWrapper::new(fastembed::EmbeddingModel::AllMiniLML6V2)?;
    let lua_vm = LuaVM::new()?;
    let file_storage = FileStorage::new("path/to/file/storage")?;
    let mut auth_manager = AuthManager::new();

    // Add a default admin user
    auth_manager.add_user("admin", vec![
        query::parser::QueryType::Select,
        query::parser::QueryType::Insert,
        query::parser::QueryType::Update,
        query::parser::QueryType::Delete,
        query::parser::QueryType::CreateNamespace,
        query::parser::QueryType::DeleteNamespace,
        query::parser::QueryType::UploadFile,
        query::parser::QueryType::ProcessFile,
        query::parser::QueryType::GenerateEmbedding,
        query::parser::QueryType::SimilaritySearch,
        query::parser::QueryType::LLMQuery,
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
            let port = port.unwrap_or(50051);
            server::api::run_server(port, query_executor).await?;
        }
        None => {
            println!("Please specify a command: cli or server");
        }
    }

    Ok(())
}