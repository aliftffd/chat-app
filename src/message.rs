use serde::{Deserialize,Serialize};
use uuid::Uuid;

#[derive(Debug,Serialize,Deserialize)]]

pub struct ChatMessage{
    pub id: Uuid,
    pub username: String,
    pub content: String,
    pub timestamp: u64,
    pub message_type: Message_type,
};


#[derive(Debug,Serialize,Deserialize)]]
pub enum Message_type{
    Text,
    Join,
    Leave,
    System,
}

impl ChatMessage{
    pub fn new(username: String, content: String, message_type: Message_type) -> Self{
        Self{
            id: Uuid::new_v4(),
            username,
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message_type,
        }
    }

    pub fn to_json(&self) -> String{
        serde_json::to_string(self).unwrap()
    }

    pub fn from_json(data: &self) -> Result<Self,serde_json::Error>{
        serde_json::from_str(data)
    }
}
