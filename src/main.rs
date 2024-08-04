mod core;
mod vector;
mod ai;
mod lua;
mod file;
mod query;
mod cli;
mod server;

use clap::{Parser, Subcommand};

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
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Cli) => {
            cli::console::run()?;
        }
        Some(Commands::Server { port }) => {
            let port = port.unwrap_or(50051);
            server::api::run_server(port).await?;
        }
        None => {
            println!("Please specify a command: cli or server");
        }
    }

    Ok(())
}