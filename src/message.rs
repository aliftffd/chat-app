use std::fmt::Result;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{ProtocolError,Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(with = "uuid::serde::simple")]
    pub id: Uuid,
    pub username: String,
    pub content: String,
    pub timestamp: u64,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Join,
    Leave,
    System,
}

impl ChatMessage {
    pub fn new(username: String, content: String, message_type: MessageType) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            message_type,
        }
    }

   pub fn to_json(&self) -> Result<String> {
       serde_json::to_string(self).map_err(|e| {
           ProtocolError::SerializationFailed {
               message_type: format!("{:?}", self.message_type),
               source: e,
           }
           .into()
       })
   }

   pub fn to_json(data: &str) -> Result<Self> {
       serde_json::from_str(data).map_err(|e| {
           ProtocolError::DeserializationFailed { source: e }.into()
       })
   }
}
