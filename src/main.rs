mod client;
mod config;
mod device;
mod error;
mod message;
mod message_store;
mod retry;
mod server;

use clap::{command, Parser, Subcommand};
use config::Config;
use std::io::Write;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "terminal-chat")]
#[command(about = "A terminal-based real-time chat application")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the chat server
    Server {
        /// Server address to bind to
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        address: String,
    },
    /// Connect to a chat server as a client
    Client {
        /// Server address to connect to
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        address: String,

        /// Enable automatic reconnection on disconnect
        #[arg(long)]
        auto_reconnect: bool,

        /// Username (skip prompt)
        #[arg(short, long)]
        username: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();
    // load config
    let config = Config::load().unwrap_or_default();
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { address } => {
            let addr = if address == "127.0.0.1:8080" {
                // Use config if default wasn't changed
                config.server.address.clone()
            } else {
                address
            };
            tracing::info!("🚀 Starting server on {}...", &addr);
            let server = server::ChatServer::new(&addr).await?;
            server.run().await?;
        }
        Commands::Client {
            address,
            auto_reconnect,
            username,
        } => {
            // Use CLI arg if provided, otherwise use config
            let addr = if address == "127.0.0.1:8080" {
                tracing::info!("Using config address: {}", config.server.address);
                config.server.address.clone()
            } else {
                tracing::info!("Using CLI address: {}", address);
                address
            };

            // Merge CLI and config settings (CLI takes priority)
            let use_auto_reconnect = auto_reconnect || config.client.auto_reconnect;
            let final_username = username.or(config.client.username.clone());

            let username = if let Some(u) = final_username {
                u
            } else {
                print!("Enter Username: ");
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            };

            if username.is_empty() {
                anyhow::bail!("Username cannot be empty!");
            }

            tracing::info!("Auto-reconnect enabled: {}", use_auto_reconnect);

            if use_auto_reconnect {
                tracing::info!("🔗 Connecting with auto-reconnect to {}...", &addr);
                client::ChatClient::run_with_auto_reconnect(addr, username).await?;
            } else {
                tracing::info!("🔗 Connecting to server at {}...", &addr);
                let client = client::ChatClient::connect(&addr).await?;
                client.run_with_username(username).await?;
            }
        }
    }

    Ok(())
}
