use thiserror::Error;

#[derive(Error,Debug)]

pub enum AppError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Command error:{0}")]
    Command(#[from] CommandError),

    #[error("System error: {0}")]
    System(#[from] SystemError),

    #[error("ID error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error,Debug)]
pub enum NetworkError{
    #[error("Connection refused to {address}")]
    ConnectionRefused { address: String },

    #[error("Connection lost (last seen {last_seen} seconds ago)")]
    ConnectionLost { last_seen: u64},

    #[error("Operation '{operation}' timed out after {timeout_secs}s")]
    Timeout {
        operation: String,
        timeout_secs: u64,
    },

    #[error({"Server unreachable at {address}: {cause}"})]
    ServerUnreachable { address: String, cause: String},
    
    #[error("Failed to bind to address {address}: {source}")]
    BindFailed{
        address: String,
        source: std::io::Error,
    },

}

#[derive(Error,Debug)]
pub enum ProtocolError {
    #[error("Failed to serialize message of type '{message_type}': {source}")]
    SerializationFailed {
        message_type: String,
        source: serde_json::Error,
    },

    #[error("Failed to deserialize message: {source")]
    DeserializationFailed {source: serde_json::Error},

    #[error("Invalid message: {reason}")]
    InvalidMessage {reason: String},

    #[error("Protocol version mistmatch: expected v{expected}, got v{got}")]
    VerisonMismatch {expected: u32, got: u32},

}

#[derive(Error,Debug)]
pub enum CommandError {
    #[error("Invalid command syntax: '{command}' - {reason}")]
    InvalidSyntax { command: String, reason: String },

    #[error("Permission denied: device '{device}' cannot execute '{command}'")]
    PermissionDenied { device: String, command: String },

    #[error("Device '{device_id}' not found")]
    DeviceNotFound { device_id: String },

    #[error("Command execution failed on '{device}': {error}")]
    ExecutionFailed { device: String, error: String },

    #[error("Command timeout: '{command}' on '{device}' did not respond")]
    CommandTimeout { device: String, command: String },
}

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("GPU not available or nvidia-smi not found")]
    GpuNotAvailable,

    #[error("Failed to spawn process '{command}': {source}")]
    ProcessSpawnFailed {
        command: String,
        source: std::io::Error,
    },

    #[error("Insufficient {resource}: required {required}, available {available}")]
    InsufficientResources {
        resource: String,
        required: String,
        available: String,
    },

    #[error("Failed to read system information: {0}")]
    SystemInfoFailed(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;

/// Extension trait for adding context to errors
pub trait Context<T> {
    fn context(self, context: impl Into<String>) -> anyhow::Result<T>;
}

impl<T, E> Context<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context(self, context: impl Into<String>) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!(e).context(context.into()))
    }
}


