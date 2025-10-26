use crate::message::{ChatMessage, MessageType};
use serde_json::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast::error::RecvError;

type SharedState = Arc<Mutex<HashMap<String, TcpStream>>>;

pub Struct ChatServer{
    listener: TcpListener,
    state: SharedState,
    sender: broadcast::Sender<String>;
}

impl ChatServer{
    pub async fn new(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let state = Arc::new(Mutex::new(HashMap::new()));
        let (sender, _) = broadcast::channel(100);

        Ok(self{
            listener,
            state,
            sender,
        })
    }

    pub async fn run(&self) -> std::io::Result<()> {
        println!("🚀 Server running on {}",self.listener.local_addr()?);

        while let Ok((stream,addr)) = self.listener.accept().await{
            println!("📡 New connection from: {}", addr);

            let state = self.state.clone();
            let sender = self.sender.clone();
            let mut receiver = sender.subscribe();

            tokio::spawn(async move{
                if let Err(e) = Self::handle_client(stream,state,sender,&mut receiver).await{
                    eprinln!("❌ Client handler error: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_client(
        mut stream: TcpStream,
        state: SharedState,
        sender: broadcast::Sender<String>,
        receiver: &mut broadcast::Receiver<String>,
    ) -> std::io::Result<()> {
        let (reader,mut writer) = stream.split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        //Read user name
        writer.write_all(b"Enter Username : ").await?;
        buf_reader.real_line(&mut line).await?;
        let username = line.trim().to_string();

        if username.is_empty(){
            writer.write_all(b"Username cannot be empty!\n").await?;
            return Ok(());
        }

        // Add client state

        {
            let mut state_lock = state.lock().await;
            state_lock.insert(username.clone(),stream);
        }

        // sent join notification
        let join_msg = ChatMessage::new(
            usernam.clone(),
            format!("{} joint the chat!",username),
            MessageType::Join;
            );

        //Welcome message
        let welcome_msg = ChatMessage::new(
            "System".to_string(),
            format!("Welcome to the chat,{}! Type 'quit' to exit",username),
            MessageType::System;
            );

        writer.write_all(format!("{}\n",welcome_msg.to_json()).as_byte()).await?;

        let mut username_clone = username.clone();
        let sender_clone = sender.clone();

        //spawn task reciver message from this client
        //

        if receive_handle = tokio::spawn(async move{
            let (reader, _) = match TcpListener::peak(&mut stream){
                Ok(_) => stream.split(),
                Err(_) => return,
            };

            let buf_reader = BufReader::new(reader);
            let mut line = String::new();

            loop{
                line.clear();
                match buf_reader.read_line(&mut line).await{
                    Ok(0) => break, // connection lost 
                    Ok(_) => {
                        let content = line.trim().to_string();

                        if content == "/quit"{
                            break;
                        }

                        if !content.is_empty(){
                            let msg = ChatMessage::new(
                                username_clone.clone(),
                                content,
                                MessageType::Text,
                                );

                            let _ = sender_clone().send(msg.to_json());
                        }
                    }
                    Err(_) => break;
                }
            }

            // send leave notification

            let leave_msg = ChatMessage::new(
                username_clone.clone(),
                format!("{} left the chat!",username_clone),
                MessageType::Leave,
                );

            let _ = sender_clone.send(leave_msg.to_json());
        });

        loop{
            match reciver.recv().await {
                Ok (message) => {
                    if let Ok(chat_msg) = ChatMessage::from_json(&message){
                            // Dont send user's own message back to them
                            if chat_msg.username != username{
                                if let Err(_) = writer.write_all(format!("{}\n",message)).await{
                                    break;
                                }
                            }
                    }
                }
                Err(RecvError::Closed) => break;
                Err(RecvError::Lagged(_)) => continue;
            }
        }

        //cleanup

        {
            let mut state_lock = state.lock().await;
            state_lock.remove(&username);
        }

        receive_handle.abort();
        println!("👋 Client disconnected: {}", username);

        Ok(());
   }
}
