mod client;
mod message;
mod server;

use clap::{command, Parser, Subcommand};

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
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { address } => {
            println!("🚀 Starting server on {}...", address);
            let server = server::ChatServer::new(&address).await?;
            server.run().await?;
        }
        Commands::Client { address } => {
            println!("🔗 Connecting to server at {}...", address);
            let client = client::ChatClient::connect(&address).await?;
            client.run().await?;
        }
    }

    Ok(())
}