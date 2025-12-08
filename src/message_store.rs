use rusqlite::{Connection, params};
use crate::message::ChatMessage;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{info, debug};

pub struct MessageStore {
    conn: Mutex<Connection>,
}

impl MessageStore{
    /// Create new message store
    pub fn new(db_path: PathBuf) -> anyhow::Result<Self>{
        // create parent directory if it does not exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        let store = Self { conn: Mutex::new(conn) };
        store.init_db()?;

        info!("Message store initialized at: {}", db_path.display());

        Ok(store)
    }

    /// Initialize database schema
    fn init_db(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages(
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                message_type TEXT NOT NULL,
                device_id TEXT,
                device_hostname TEXT,
                device_os TEXT,
                device_type TEXT
                )",
            [],
        )?;

        // Create index for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON messages(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_device ON messages(device_id)",
            [],
        )?;

        debug!("Database schema initialized");
        Ok(())
    }
    
    /// Store a message
    pub fn store(&self, msg: &ChatMessage) -> anyhow::Result<()> {
        let device_id = msg.device.as_ref().map(|d| d.device_id.as_str());
        let device_hostname = msg.device.as_ref().map(|d| d.hostname.as_str());
        let device_os = msg.device.as_ref().map(|d| d.os.as_str());
        let device_type = msg.device.as_ref().map(|d| format!("{:?}", d.device_type));

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (id, username, content , timestamp, message_type,device_id,
            device_hostname,device_os,device_type) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                msg.id.to_string(),
                &msg.username,
                &msg.content,
                msg.timestamp as i64,
                format!("{:?}", msg.message_type),
                device_id,
                device_hostname,
                device_os,
                device_type.as_deref(),
            ],
        )?;

        debug!("Stored message: {} from {}", msg.id,msg.username);
        Ok(())
    }

    /// Retrieve recent messages
    pub fn recent(&self, count: usize) -> anyhow::Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, content, timestamp, message_type,
                    device_id, device_hostname, device_os, device_type
             FROM messages
             ORDER BY timestamp DESC
             LIMIT ?1"
        )?;

        let messages = stmt.query_map(params![count], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        })?;

        let mut result = Vec::new();
        for msg in messages {
            if let Ok(data) = msg {
                if let Some(chat_msg) = Self::row_to_message(data) {
                    result.push(chat_msg);
                }
            }
        }

        result.reverse(); // Oldest first

        debug!("Retrieved {} recent messages", result.len());
        Ok(result)
    }

    pub fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<ChatMessage>>{
        let search_pattern = format!("%{}%", query);

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, content, timestamp, message_type,
                    device_id, device_hostname, device_os, device_type
             FROM messages
             WHERE content LIKE ?1 OR username LIKE ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
            )?;

        let messages = stmt.query_map(params![search_pattern, limit], |row|{
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                ))
        })?;

        let mut result = Vec::new();
        for msg in messages {
            if let Ok(data) = msg {
                if let Some(chat_msg) = Self::row_to_message(data) {
                    result.push(chat_msg);
                }
            }
        }

        result.reverse();

        debug!("Found {} messages matching '{}'", result.len(), query);
        Ok(result)
    }

    pub fn count(&self) -> anyhow::Result<usize>{
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn row_to_message(
        data: (String, String, String, i64, String, Option<String>, Option<String>,
            Option<String>,Option<String>)
        ) -> Option<ChatMessage> {
            use crate::message::MessageType;
            use crate::device::{DeviceInfo, DeviceType, DeviceStatus};
            use uuid::Uuid;
            
            let (id, username, content , timestamp, msg_type_str,
                device_id, device_hostname, device_os, _device_type) = data;
            let id = Uuid::parse_str(&id).ok()?;

            let message_type = match msg_type_str.as_str() {
                "Text" => MessageType::Text,
                "Join" => MessageType::Join,
                "Leave" => MessageType::Leave,
                "System" => MessageType::System,
                "DeviceRegistration" => MessageType::DeviceRegistration,
                "DeviceList" => MessageType::DeviceList,
                _ => MessageType::Text,
            };

            let device = if let (Some(dev_id), Some(hostname), Some(os)) = 
                (device_id,device_hostname,device_os) {
                Some(DeviceInfo {
                    device_id: dev_id,
                    hostname,
                    os,
                    device_type: DeviceType::Unknown,
                    status: DeviceStatus::Online,
                    last_seen: timestamp as u64,
                })
            } else {
                None
            };

            Some(ChatMessage {
                id,
                username,
                content,
                timestamp: timestamp as u64,
                message_type,
                device,
            })
    }
    pub fn default_path() -> PathBuf{
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("chat-app")
            .join("messages.db")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_store() {
        let temp_db = PathBuf::from("tmp/test_message.db");
        let _ = std::fs::remove_file(&temp_db);

        let store = MessageStore::new(temp_db.clone()).unwrap();

        let msg = ChatMessage::new(
            "test_user".to_string(),
            "test message".to_string(),
            crate::message::MessageType::Text,
        );

        store.store(&msg).unwrap();

        let messages = store.recent(10).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "test message");

        let _ = std::fs::remove_file(&temp_db);

    }
}
