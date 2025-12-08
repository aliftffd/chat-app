use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,      // "lab-pc", "arch-main", etc.
    pub hostname: String,        // System hostname
    pub os: String,              // "Linux", "Windows", etc.
    pub device_type: DeviceType, // Role of device
    pub status: DeviceStatus,    // Current status
    pub last_seen: u64,          // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Server,        // Main server/controller
    Workstation,   // Heavy compute (Lab PC)
    Laptop,        // Development laptop
    Edge,          // Edge device (Jetson)
    Unknown,       // Unknown type
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Busy,
    Idle,
}

impl DeviceInfo {
    /// Create new device info from system
    pub fn new(device_id: Option<String>) -> Self {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        let os = std::env::consts::OS.to_string();
        
        // Use provided device_id or generate from hostname
        let device_id = device_id.unwrap_or_else(|| {
            hostname.to_lowercase().replace(" ", "-")
        });

        let device_type = Self::detect_device_type(&device_id, &os);

        Self {
            device_id,
            hostname,
            os,
            device_type,
            status: DeviceStatus::Online,
            last_seen: Self::current_timestamp(),
        }
    }

    /// Detect device type from ID and OS
    fn detect_device_type(device_id: &str, os: &str) -> DeviceType {
        let id_lower = device_id.to_lowercase();
        
        if id_lower.contains("server") {
            DeviceType::Server
        } else if id_lower.contains("lab") || id_lower.contains("workstation") {
            DeviceType::Workstation
        } else if id_lower.contains("jetson") || id_lower.contains("nano") || id_lower.contains("edge") {
            DeviceType::Edge
        } else if id_lower.contains("laptop") || os == "linux" || os == "macos" {
            DeviceType::Laptop
        } else {
            DeviceType::Unknown
        }
    }

    /// Get current Unix timestamp
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Get status indicator
    pub fn status_indicator(&self) -> &str {
        match self.status {
            DeviceStatus::Online => "🟢",
            DeviceStatus::Offline => "🔴",
            DeviceStatus::Busy => "🟡",
            DeviceStatus::Idle => "⚪",
        }
    }

    /// Format device type as string
    pub fn type_str(&self) -> &str {
        match self.device_type {
            DeviceType::Server => "Server",
            DeviceType::Workstation => "Workstation",
            DeviceType::Laptop => "Laptop",
            DeviceType::Edge => "Edge",
            DeviceType::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} ({}) - {}",
            self.status_indicator(),
            self.device_id,
            self.os,
            self.type_str(),
            if self.status == DeviceStatus::Online {
                "Online"
            } else {
                "Offline"
            }
        )
    }
}
