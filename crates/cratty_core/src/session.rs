use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "session-{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    Local,
    Ssh,
    Serial,
}

/// Serializable snapshot for session recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_type: SessionType,
    pub profile_name: String,
    pub working_directory: Option<String>,
}

/// Core session trait implemented by all backends.
pub trait Session: Send {
    fn id(&self) -> SessionId;
    fn title(&self) -> String;
    fn session_type(&self) -> SessionType;

    /// Write user input to the session.
    fn write(&self, data: &[u8]) -> anyhow::Result<()>;

    /// Resize the terminal.
    fn resize(&self, cols: u16, rows: u16) -> anyhow::Result<()>;

    /// Gracefully close the session.
    fn close(&mut self) -> anyhow::Result<()>;

    fn is_alive(&self) -> bool;

    /// Snapshot for session recovery.
    fn serialize_state(&self) -> anyhow::Result<SessionState>;
}
