use crate::message::ChatMessage;
use crate::error::{NetworkError, Result};
use crate::device::DeviceInfo;  // NEW
use colored::*;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use anyhow::Context;

pub struct ChatClient {
    stream: TcpStream,
}

impl ChatClient {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn show_banner() {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!("{}", "╔════════════════════════════════════════════════════════╗".cyan());
        println!("{}", format!("║        💬 Terminal Chat v{}                     ║", Self::VERSION).cyan());
        println!("{}", "║        Real-time communication & device control       ║".cyan());
        println!("{}", "╚════════════════════════════════════════════════════════╝".cyan());
        println!();
    }

    fn show_help() {
        println!();
        println!("{}", "╔════════════════════════════════════════════════════════╗".cyan());
        println!("{}", "║                  Available Commands                    ║".cyan());
        println!("{}", "╚════════════════════════════════════════════════════════╝".cyan());
        println!();
        println!("  {}  - Show this help message", "/help".yellow());
        println!("  {}  - Clear the screen", "/clear".yellow());
        println!("  {}  - List all connected devices", "/devices".yellow());
        println!("  {}  - Show last N messages (default: 20)", "/history [N]".yellow());
        println!("  {}  - Exit the chat", "/quit".yellow());
        println!();
        println!("{}", "  Tip: Recent messages are shown when you connect".dimmed());
        println!();
    }

    fn clear_screen(){
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().ok();
    }

    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await.map_err(|_e| {
            NetworkError::ConnectionRefused {
                address: addr.to_string(),
            }
        })?;

        info!("✅ Connected to server at {}", addr);
        println!("✅ Connected to server at {}", addr);

        Ok(Self { stream })
    }


    pub async fn run_with_username(self, username: String) -> anyhow::Result<()> {
        // Show banner
        Self::show_banner();

        let (reader, mut writer) = self.stream.into_split();
        let mut buf_reader = BufReader::new(reader);

        // Send username to server
       writer.write_all(format!("{}\n", username).as_bytes()).await
           .context("Failed to send username to server")?;
       writer.flush().await
           .context("Failed to flush after sending username")?;
       //senmd device info to server
       let device_info = DeviceInfo::new(None);
       let device_json = serde_json::to_string(&device_info)
           .context("Failed to serialize device info")?;
        writer.write_all(format!("{}\n", device_json).as_bytes()).await
            .context("Failed to send device info to server")?;
        writer.flush().await
            .context("Failed to flush after sending device info")?;
        info!("Device registered: {} {} ({})",device_info.device_id, device_info.os, device_info.type_str());
        let (tx, mut rx) = mpsc::channel::<String>(100);

        // Spawn task for reading user input
        let input_handle = tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut stdin_reader = BufReader::new(stdin);
            let mut line = String::new();

            // Show initial prompt
            if let Err(e) = Self::show_prompt() {
                error!("Failed to show prompt: {}", e);
            }

            loop {
                line.clear();

                match stdin_reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("EOF received from stdin");
                        break;
                    }
                    Ok(_) => {
                        let content = line.trim().to_string();
                        if content == "/quit" {
                            if let Err(e) = tx.send(content).await {
                                error!("Failed to send quit command: {}", e);
                            }
                            break;
                        } else if content == "/help" {
                            Self::show_help();
                            if let Err(e) = Self::show_prompt(){
                                error!("Failed to show Prompt : {}", e);
                            }
                            continue;
                        } else if content == "/clear"{
                            Self::clear_screen();
                            if let Err(e) = Self::show_prompt() {
                                error!("Failed to show prompt: {}",e);
                            }
                            continue;
                        }
                        if !content.is_empty() {
                            if let Err(e) = tx.send(content).await {
                                error!("Failed to send message: {}", e);
                                break;
                            }
                        }

                        // Show prompt for next message
                        if let Err(e) = Self::show_prompt() {
                            error!("Failed to show prompt: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn task for receiving messages from server
        let receive_handle = tokio::spawn(async move {
            let mut line = String::new();

            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("\n{}", "🔌 Connection closed by server".red());
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        match ChatMessage::from_json(trimmed) {
                            Ok(chat_msg) => {
                                // Clear current line and print message
                                print!("\r\x1B[K");
                                Self::display_message(&chat_msg);

                                // Re-print prompt
                                if let Err(e) = Self::show_prompt() {
                                    error!("Failed to show prompt: {}", e);
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse message: {} - raw: {}", e, trimmed);
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Error reading from server: {}", e);
                        println!("\n❌ Error reading from server: {}", e);
                        break;
                    }
                }
            }
        });

        // Handle sending messages
        while let Some(message) = rx.recv().await {
            if message == "/quit" {
                info!("User requested quit");
                break;
            }

            if let Err(e) = writer.write_all(format!("{}\n", message).as_bytes()).await {
                error!("❌ Error sending message: {}", e);
                eprintln!("❌ Error sending message: {}", e);
                break;
            }

            // Flush to ensure message is sent immediately
            if let Err(e) = writer.flush().await {
                error!("❌ Error flushing message: {}", e);
                eprintln!("❌ Error flushing message: {}", e);
                break;
            }
        }

        // Cleanup: abort background tasks
        input_handle.abort();
        receive_handle.abort();

        println!("👋 Disconnected from server");
        info!("Client disconnected cleanly");

        Ok(())
    }

    fn show_prompt() -> io::Result<()> {
        print!("{}", "> ".green());
        io::stdout().flush()
    }

    fn display_message(msg: &ChatMessage) {
        let timestamp = chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "??:??:??".to_string());

        // Device info display
        let device_str = if let Some(ref dev) = msg.device {
            format!("[{}] ", dev.device_id.cyan())
        } else {
            String::new()
        };

        match msg.message_type {
            crate::message::MessageType::Text => {
                println!(
                    "[{}] {}{}: {}",
                    timestamp.dimmed(),
                    device_str,
                    msg.username.blue(),
                    msg.content
                );
            }
            crate::message::MessageType::Join => {
                println!(
                    "[{}] {} {}{}",
                    timestamp.dimmed(),
                    "➡️".green(),
                    device_str,
                    msg.content.yellow()
                );
            }
            crate::message::MessageType::Leave => {
                println!(
                    "[{}] {} {}{}",
                    timestamp.dimmed(),
                    "⬅️".red(),
                    device_str,
                    msg.content.yellow()
                );
            }
            crate::message::MessageType::System => {
                println!(
                    "[{}] {} {}",
                    timestamp.dimmed(),
                    "⚡".cyan(),
                    msg.content.cyan()
                );
            }
            _ => {
                // Handle other message types
                println!(
                    "[{}] {}{}",
                    timestamp.dimmed(),
                    device_str,
                    msg.content
                );
            }
        }
    }

    /// Connect with automatic retry on failure
    pub async fn connect_with_retry(addr: &str) -> anyhow::Result<Self> {
        use crate::retry::{retry_with_backoff, RetryConfig};

        let addr = addr.to_string();
        let config = RetryConfig::default();

        retry_with_backoff(
            move || {
                let addr = addr.clone();
                Box::pin(async move {
                    Self::connect(&addr).await
                })
            },
            config,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect after retries: {}", e))
    }

    /// Run with automatic reconnection on disconnect
    pub async fn run_with_auto_reconnect(addr: String, username: String) -> anyhow::Result<()> {
        Self::show_banner();
        loop {
            info!("Attempting to connect to {}...", addr);

            match Self::connect_with_retry(&addr).await {
                Ok(client) => {
                    info!("Connected successfully, starting chat...");

                    match client.run_internal(&username).await {
                        Ok(_) => {
                            info!("User quit intentionally");
                            break; // User quit with /quit
                        }
                        Err(e) => {
                            error!("❌ Disconnected: {}. Reconnecting...", e);
                            println!("\n⚠️  Connection lost. Reconnecting...\n");
                            continue; // Reconnect automatically
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Internal run method that can be restarted
    async fn run_internal(self, username: &str) -> anyhow::Result<()> {
        let (reader, mut writer) = self.stream.into_split();
        let mut buf_reader = BufReader::new(reader);

        // Send username to server
        writer.write_all(format!("{}\n", username).as_bytes()).await
            .context("Failed to send username to server")?;
        writer.flush().await
            .context("Failed to flush after sending username")?;

        // Send device info to server
        let device_info = DeviceInfo::new(None);
        let device_json = serde_json::to_string(&device_info)
            .context("Failed to serialize device info")?;
        writer.write_all(format!("{}\n", device_json).as_bytes()).await
            .context("Failed to send device info to server")?;
        writer.flush().await
            .context("Failed to flush after sending device info")?;
        info!("Device registered: {} {} ({})", device_info.device_id, device_info.os, device_info.type_str());

        let (tx, mut rx) = mpsc::channel::<String>(100);
        
        // NEW: Channel to signal connection errors
        let (error_tx, mut error_rx) = mpsc::channel::<anyhow::Error>(1);

        // Spawn task for reading user input
        let input_handle = tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut stdin_reader = BufReader::new(stdin);
            let mut line = String::new();

            if let Err(e) = Self::show_prompt() {
                error!("Failed to show prompt: {}", e);
            }

            loop {
                line.clear();

                match stdin_reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let content = line.trim().to_string();
                        if content == "/quit" {
                            if let Err(e) = tx.send(content).await {
                                error!("Failed to send quit command: {}", e);
                            }
                            break;
                        } else if content == "/help" {
                            Self::show_help();
                            if let Err(e) = Self::show_prompt() {
                                error!("Failed to show prompt {}",e);
                            }
                            continue;
                        } else if content == "/clear" {
                            Self::clear_screen();
                            if let Err(e) = Self::show_prompt() {
                                error!("Failed to show_prompt {}",e);
                            }
                            continue;
                        }
                        if !content.is_empty() {
                            if let Err(e) = tx.send(content).await {
                                error!("Failed to send message: {}", e);
                                break;
                            }
                        }

                        if let Err(e) = Self::show_prompt() {
                            error!("Failed to show prompt: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn task for receiving messages from server
        let error_tx_clone = error_tx.clone();
        let receive_handle = tokio::spawn(async move {
            let mut line = String::new();

            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("\n{}", "🔌 Connection closed by server".red());
                        // Signal connection loss
                        let _ = error_tx_clone.send(anyhow::anyhow!("Connection closed by server")).await;
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        match ChatMessage::from_json(trimmed) {
                            Ok(chat_msg) => {
                                print!("\r\x1B[K");
                                Self::display_message(&chat_msg);

                                if let Err(e) = Self::show_prompt() {
                                    error!("Failed to show prompt: {}", e);
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse message: {} - raw: {}", e, trimmed);
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Error reading from server: {}", e);
                        println!("\n❌ Error reading from server: {}", e);
                        // Signal connection error
                        let _ = error_tx_clone.send(anyhow::anyhow!("Read error: {}", e)).await;
                        break;
                    }
                }
            }
        });

        // Handle sending messages
        loop {
            tokio::select! {
                // Check for messages to send
                Some(message) = rx.recv() => {
                    if message == "/quit" {
                        info!("User requested quit");
                        input_handle.abort();
                        receive_handle.abort();
                        println!("👋 Disconnected from server");
                        return Ok(()); // Normal exit
                    }

                    if let Err(e) = writer.write_all(format!("{}\n", message).as_bytes()).await {
                        error!("❌ Error sending message: {}", e);
                        input_handle.abort();
                        receive_handle.abort();
                        return Err(anyhow::anyhow!("Failed to send message: {}", e));
                    }

                    if let Err(e) = writer.flush().await {
                        error!("❌ Error flushing message: {}", e);
                        input_handle.abort();
                        receive_handle.abort();
                        return Err(anyhow::anyhow!("Failed to flush message: {}", e));
                    }
                }
                
                // Check for connection errors
                Some(err) = error_rx.recv() => {
                    error!("Connection error detected: {}", err);
                    input_handle.abort();
                    receive_handle.abort();
                    return Err(err); // Return error to trigger reconnect
                }
                
                // If both channels close, exit
                else => {
                    info!("All channels closed");
                    input_handle.abort();
                    receive_handle.abort();
                    println!("👋 Disconnected from server");
                    return Ok(());
                }
            }
        }
    }
}
