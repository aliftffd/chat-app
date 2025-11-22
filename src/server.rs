use crate::message::{ChatMessage, MessageType};
use crate::error::{NetworkError, Result};
use crate::device::DeviceInfo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn, error, debug};
use anyhow::Context;

type SharedState = Arc<Mutex<HashMap<String, DeviceInfo>>>;

pub struct ChatServer {
    listener: TcpListener,
    state: SharedState,
    sender: broadcast::Sender<String>,
}

impl ChatServer {
    pub async fn new(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            NetworkError::BindFailed {
                address: addr.to_string(),
                source: e,
            }
        })?;

        let state = Arc::new(Mutex::new(HashMap::new()));
        let (sender, _) = broadcast::channel(100);

        info!("Server initialized on {}", addr);

        Ok(Self {
            listener,
            state,
            sender,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let local_addr = self.listener.local_addr()
            .context("Failed to get local address")?;
        
        info!("🚀 Server running on {}", local_addr);

        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("📡 New connection from: {}", addr);

                    let state = self.state.clone();
                    let sender = self.sender.clone();
                    let mut receiver = sender.subscribe();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, state, sender, &mut receiver).await {
                            error!("Client handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_client(
        stream: TcpStream,
        state: SharedState,
        sender: broadcast::Sender<String>,
        receiver: &mut broadcast::Receiver<String>,
    ) -> anyhow::Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        // Read username
        buf_reader.read_line(&mut line).await
            .context("Failed to read username")?;
        let username = line.trim().to_string();

        if username.is_empty() {
            writer.write_all(b"Username cannot be empty!\n").await?;
            return Ok(());
        }

        // Read device info
        line.clear();
        buf_reader.read_line(&mut line).await
            .context("Failed to read device info")?;
        
        let device_info: DeviceInfo = serde_json::from_str(line.trim())
            .unwrap_or_else(|_| DeviceInfo::new(None));

        info!(
            "✅ User '{}' joined from device: {} {} ({})",
            username, device_info.device_id, device_info.os, device_info.type_str()
        );

        // Register device
        {
            let mut state_lock = state.lock().await;
            state_lock.insert(device_info.device_id.clone(), device_info.clone());
        }

        // Send join notification with device info
        let join_msg = ChatMessage::new(
            username.clone(),
            format!("{} joined from {}!", username, device_info.device_id),
            MessageType::Join,
        )
        .with_device(device_info.clone());

        if let Ok(json) = join_msg.to_json() {
            let _ = sender.send(json);
        }

        // Welcome message
        let welcome_msg = ChatMessage::new(
            "System".to_string(),
            format!("Welcome to the chat, {}! Type '/quit' to exit", username),
            MessageType::System,
        );

        if let Ok(json) = welcome_msg.to_json() {
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            writer.flush().await?;
        }

        let username_clone = username.clone();
        let sender_clone = sender.clone();
        let device_id = device_info.device_id.clone();
        let device_info_clone = device_info.clone();

        // Spawn task to receive messages from this client
        let receive_handle = tokio::spawn(async move {
            let mut line = String::new();

            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("Client connection closed");
                        break;
                    }
                    Ok(_) => {
                        let content = line.trim().to_string();

                        if content == "/quit" {
                            debug!("Client requested quit");
                            break;
                        }

                        // Handle /devices command
                        if content == "/devices" {
                            // Will be handled by sending a special message
                            let msg = ChatMessage::new(
                                username_clone.clone(),
                                "/devices".to_string(),
                                MessageType::System,
                            )
                            .with_device(device_info_clone.clone());

                            if let Ok(json) = msg.to_json() {
                                let _ = sender_clone.send(json);
                            }
                            continue;
                        }

                        if !content.is_empty() {
                            let msg = ChatMessage::new(
                                username_clone.clone(),
                                content.clone(),
                                MessageType::Text,
                            )
                            .with_device(device_info_clone.clone());

                            info!("📨 Message from {} [{}]: {}", username_clone, device_id, content);
                            
                            if let Ok(json) = msg.to_json() {
                                if let Err(e) = sender_clone.send(json) {
                                    error!("Failed to broadcast message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading from client: {}", e);
                        break;
                    }
                }
            }

            // Send leave notification
            let leave_msg = ChatMessage::new(
                username_clone.clone(),
                format!("{} left the chat!", username_clone),
                MessageType::Leave,
            )
            .with_device(device_info_clone.clone());

            info!("👋 User '{}' [{}] is leaving", username_clone, device_id);
            
            if let Ok(json) = leave_msg.to_json() {
                let _ = sender_clone.send(json);
            }
        });

        // Broadcast messages to this client
        loop {
            match receiver.recv().await {
                Ok(message) => {
                    if let Ok(chat_msg) = ChatMessage::from_json(&message) {
                        // Handle /devices command
                        if chat_msg.content == "/devices" 
                            && chat_msg.message_type == MessageType::System 
                            && chat_msg.username == username {
                            // Send device list to this client only
                            let state_lock = state.lock().await;
                            let device_list: Vec<String> = state_lock
                                .values()
                                .map(|d| format!("{}", d))
                                .collect();
                            
                            let devices_msg = format!("\n📱 Connected Devices ({}):\n{}\n",
                                device_list.len(),
                                device_list.join("\n")
                            );
                            
                            if let Err(_) = writer.write_all(devices_msg.as_bytes()).await {
                                break;
                            }
                            let _ = writer.flush().await;
                            continue;
                        }

                        // Don't send user's own message back to them
                        if chat_msg.username != username {
                            if let Err(_) = writer.write_all(format!("{}\n", message).as_bytes()).await {
                                break;
                            }
                            let _ = writer.flush().await;
                        }
                    }
                }
                Err(RecvError::Closed) => {
                    debug!("Broadcast channel closed");
                    break;
                }
                Err(RecvError::Lagged(n)) => {
                    warn!("Client lagged behind by {} messages", n);
                    continue;
                }
            }
        }

        // Cleanup
        {
            let mut state_lock = state.lock().await;
            state_lock.remove(&device_info.device_id);
        }

        receive_handle.abort();
        info!("👋 Client disconnected: {} [{}]", username, device_info.device_id);

        Ok(())
    }
}
