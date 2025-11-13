mod client;
mod message;
mod server;
mod error;

use clap::{command, Parser, Subcommand};
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
    },
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter:;from_default_env()
                .add_directive(tracing::Level::INFO.into())
            )
            .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { address } => {
            tracing::info!("🚀 Starting server on {}...", address);
            let server = server::ChatServer::new(&address).await?;
            server.run().await?;
        }
        Commands::Client { address } => {
            tracing::info!("🔗 Connecting to server at {}...", address);
            let client = client::ChatClient::connect(&address).await?;
            client.run().await?;
        }
    }

    Ok(())
}
