use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BillyBassCommand {
    /// Base64 encoded audio data
    pub audio: String,

    /// Audio format (e.g., "wav")
    pub audio_format: String,

    /// Simple sequence of mouth movements
    pub mouth_movements: Vec<MouthMovement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouthMovement {
    /// Time offset from audio start (ms)
    pub start_time_ms: u32,

    /// Direction (true = opening, false = closing)
    pub is_opening: bool,

    /// Speed (0-255)
    pub speed: u8,

    /// Duration to hold this movement (ms)
    pub duration_ms: u32,
}
