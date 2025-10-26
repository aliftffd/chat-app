use crate::message::ChatMessage;
use colored::*;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub struct ChatClient {
    stream: TcpStream,
}

impl ChatClient {
    pub async fn connect(addr: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        println!("✅ Connected to server at {}", addr);
        Ok(Self { stream })
    }

    pub async fn run(self) -> std::io::Result<()> {
        let (reader, mut writer) = self.stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        
        // Get username from user
        print!("Enter Username: ");
        io::stdout().flush().unwrap();
        
        let stdin = std::io::stdin();
        let mut username = String::new();
        stdin.read_line(&mut username).unwrap();
        let username = username.trim().to_string();
        
        if username.is_empty() {
            println!("Username cannot be empty!");
            return Ok(());
        }
        
        // Send username to server
        writer.write_all(format!("{}\n", username).as_bytes()).await?;
        writer.flush().await?;
        
        let (tx, mut rx) = mpsc::channel::<String>(100);

        // Spawn task for reading user input
        let input_handle = tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut stdin_reader = BufReader::new(stdin);
            let mut line = String::new();

            // Show initial prompt
            print!("{}", "> ".green());
            io::stdout().flush().unwrap();

            loop {
                line.clear();

                match stdin_reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let content = line.trim().to_string();
                        if content == "/quit" {
                            let _ = tx.send(content).await;
                            break;
                        }
                        if !content.is_empty() {
                            let _ = tx.send(content).await;
                        }
                        // Show prompt for next message
                        print!("{}", "> ".green());
                        io::stdout().flush().unwrap();
                    }
                    Err(_) => break,
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
                        if let Ok(chat_msg) = ChatMessage::from_json(trimmed) {
                            // Clear current line and print message
                            print!("\r\x1B[K"); // Clear line
                            Self::display_message(&chat_msg);
                            // Re-print prompt
                            print!("{}", "> ".green());
                            io::stdout().flush().unwrap();
                        } else if !trimmed.is_empty() {
                            // Only show raw text if it's not JSON (for debugging)
                            // Most messages should be parsed as ChatMessage
                        }
                    }
                    Err(e) => {
                        eprintln!("\n❌ Error reading from server: {}", e);
                        break;
                    }
                }
            }
        });

        // Handle sending messages
        while let Some(message) = rx.recv().await {
            if message == "/quit" {
                break;
            }

            if let Err(e) = writer.write_all(format!("{}\n", message).as_bytes()).await {
                eprintln!("❌ Error sending message: {}", e);
                break;
            }

            // Flush to ensure message is sent immediately
            if let Err(e) = writer.flush().await {
                eprintln!("❌ Error flushing message: {}", e);
                break;
            }
        }

        // Cleanup: abort background tasks
        input_handle.abort();
        receive_handle.abort();
        println!("👋 Disconnected from server");

        Ok(())
    }

    fn display_message(msg: &ChatMessage) {
        let timestamp = chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
            .unwrap()
            .format("%H:%M:%S")
            .to_string();

        match msg.message_type {
            crate::message::MessageType::Text => {
                println!(
                    "[{}] {}: {}",
                    timestamp.dimmed(),
                    msg.username.blue(),
                    msg.content
                );
            }
            crate::message::MessageType::Join => {
                println!(
                    "[{}] {} {}",
                    timestamp.dimmed(),
                    "➡️".green(),
                    msg.content.yellow()
                );
            }
            crate::message::MessageType::Leave => {
                println!(
                    "[{}] {} {}",
                    timestamp.dimmed(),
                    "⬅️".red(),
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
        }
    }
}
