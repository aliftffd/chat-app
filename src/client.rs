use crate::message::ChatMessage;
use colored::*;
use std::io::{self,Write};
use tokio::io::{AsyncBuffReadExt,BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub struct ChatClient {
    stream: TcpStream,
}

impl ChatClient{
    pub async fn connect(addr: &str) -> std::io::Result<self>{
        let stream = TcpStream::connect(addr).await?;
        println!("✅ Connected to server at {}",addr);
        Ok(self, {stream})
    }

    pub async fn run(&mut self) -> std::io::Result<()>{
        let (reader, mut writer) = self.stream.split();
        let mut buf_reader = BufReader::new(reader);

        let (tx,mut rx) = mpsc::channel::<String>(100);

        //Spawn task for reading user input
        let input_handle = tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut stdin_reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                print!("{}", "> ".green());
                io::stdout().flush().unwrap();
                line.clear();

                match stdin_reader.read_line(&mut line).await{
                    Ok(0) => break,// EOF 
                    Ok(_) => {
                        let content = line.trim().to_string();
                        if content = "/quit" {
                            let _ = tx.send(content).await;
                            break;
                        }
                        if !content.is_empty(){
                            let _ = tx.send(content).await;
                        }
                    }
                    Err(_) => break,
                }

            }
        });

        // Spawn task for receiving messages from server 
        let receive_handle = tokio::spawn(async move {
            let (reader, _ ) = buf_reader.into_inner().split();
            let buf_reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match buf_reader.readline(&mut line).await {
                    Ok(0) => {
                        println!("{}", "🔌 Connection closed by server".red());
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if let Ok(chat_msg) = ChatMessage::from_json(trimmed){
                            Self::display_message(&chat_msg);
                        }else if !trimmed.is_empty(){
                            println!("{}",trimmed);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Error reading from server: {}",e);
                        break;
                    }
                }
            }
        });

        //Handle sending messages 
        while let Some(messages) = rx.recv().await{
            if message = "/quit"{
                break;
            }
            
            if let Err(e) = writer.write_all(format!("{}\n",message).as_byte()).await {
                eprintln!("❌ Error sending message: {}",e);
                break;
            }

            input_handle.abort();
            receive_handle.abort();

            println!("👋 Disconnected from server");
            Ok(());
        }

        fn display_message(msg: &ChatMessage){
            let timestamp = chrono::DateTime::from_timestamp(msg.timestamp as i64,0)
                .unwrap()
                .format("%H:%M:%S")
                .to_string();

            match msg.message_type{
                crate::message::MessageType::Text => {
                    println!(
                        "{}{}{}",
                        timestamp.dimmed(),
                        msg.username.blue(),
                        msg.content
                    );
                }

                crate::message::MessageType::Join => {
                    println!(
                        "{} {} {}",
                        timestamp.dimmed(),
                        "➡️".green(),
                        msg.content.yellow()
                    );
                }
                crate::message::MessageType::Leave => {
                    println!(
                        "{} {} {}",
                        timestamp.dimmed(),
                        "⬅️".red(),
                        msg.content.yellow()
                    );
                }
                 crate::message::MessageType::System => {
                    println!(
                        "{} {} {}",
                        timestamp.dimmed(),
                        "⚡".cyan(),
                        msg.content.cyan()
                    );
                 }
            }
        }
    }
}

