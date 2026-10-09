use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_MESSAGE_BYTES: usize = 16 * 1024;
pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Default)]
pub struct SequenceGate { last: u64 }

impl SequenceGate {
    pub fn accept(&mut self, sequence: u64) -> bool {
        if sequence <= self.last { return false; }
        self.last = sequence;
        true
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Stick {
    pub x: f32,
    pub y: f32,
}

impl Stick {
    pub fn validated(self) -> Option<Self> {
        if self.x.is_finite() && self.y.is_finite() && (-1.0..=1.0).contains(&self.x) && (-1.0..=1.0).contains(&self.y) {
            Some(self)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GamepadButtons {
    pub a: bool, pub b: bool, pub x: bool, pub y: bool,
    pub lb: bool, pub rb: bool, pub start: bool, pub back: bool,
    pub l3: bool, pub r3: bool,
    pub dpad_up: bool, pub dpad_down: bool, pub dpad_left: bool, pub dpad_right: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Triggers { pub left: f32, pub right: f32 }

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GamepadState {
    pub sequence: u64,
    pub left_stick: Stick,
    pub right_stick: Stick,
    pub buttons: GamepadButtons,
    pub triggers: Triggers,
}

impl GamepadState {
    pub fn is_valid(&self) -> bool {
        self.left_stick.validated().is_some()
            && self.right_stick.validated().is_some()
            && self.triggers.left.is_finite() && (0.0..=1.0).contains(&self.triggers.left)
            && self.triggers.right.is_finite() && (0.0..=1.0).contains(&self.triggers.right)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ClientMessage {
    Authenticate { token: String, protocol_version: u16 },
    Heartbeat { client_time: u64 },
    GamepadState { #[serde(flatten)] state: GamepadState },
    MouseMove { sequence: u64, dx: i32, dy: i32 },
    MouseButton { sequence: u64, button: MouseButton, pressed: bool },
    MouseScroll { sequence: u64, delta: i32 },
    TextInput { sequence: u64, text: String },
    Key { sequence: u64, key: RemoteKey, pressed: bool, modifiers: Vec<Modifier> },
    Media { sequence: u64, action: MediaAction },
    EmergencyStop { sequence: u64 },
}

impl ClientMessage {
    pub fn sequence(&self) -> Option<u64> {
        match self {
            Self::GamepadState { state } => Some(state.sequence),
            Self::MouseMove { sequence, .. } | Self::MouseButton { sequence, .. }
            | Self::MouseScroll { sequence, .. } | Self::TextInput { sequence, .. }
            | Self::Key { sequence, .. } | Self::Media { sequence, .. }
            | Self::EmergencyStop { sequence } => Some(*sequence),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton { Left, Right, Middle }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier { Ctrl, Alt, Shift, Meta }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteKey { Enter, Escape, Backspace, Tab, ArrowUp, ArrowDown, ArrowLeft, ArrowRight }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaAction { VolumeUp, VolumeDown, Mute, PlayPause, NextTrack, PreviousTrack }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ServerMessage {
    Authenticated { session_id: String, server_name: String },
    HeartbeatAck { client_time: u64, server_time: u64 },
    Diagnostic { sequence: u64, gamepad: GamepadState, backend: String },
    Error { code: String, message: String },
    Disconnected { reason: String },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub addresses: Vec<String>,
    pub port: u16,
    pub connected: bool,
    pub gamepad_backend: String,
    pub protocol_version: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PairRequest { pub code: String, pub client_name: String }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairResponse { pub token: String, pub expires_in_seconds: u64 }

pub type DiagnosticMetadata = BTreeMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_range_sticks_and_triggers() {
        let mut state = GamepadState::default();
        assert!(state.is_valid());
        state.left_stick.x = 1.01;
        assert!(!state.is_valid());
        state.left_stick.x = 0.0;
        state.triggers.right = -0.1;
        assert!(!state.is_valid());
    }

    #[test]
    fn rejects_malformed_messages() {
        let raw = r#"{"type":"mouse_move","sequence":1,"dx":"not-a-number"}"#;
        assert!(serde_json::from_str::<ClientMessage>(raw).is_err());
    }

    #[test]
    fn rejects_old_sequences_and_new_session_starts_clean() {
        let mut first_session = SequenceGate::default();
        assert!(first_session.accept(10));
        assert!(!first_session.accept(9));
        assert!(!first_session.accept(10));
        let mut reconnected_session = SequenceGate::default();
        assert!(reconnected_session.accept(1));
    }
}

