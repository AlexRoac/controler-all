use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use remotepad_protocol::{MediaAction, Modifier, MouseButton, RemoteKey};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("input initialization failed: {0}")]
    Initialization(String),
    #[error("input operation failed: {0}")]
    Operation(String),
}

pub struct InputController {
    enigo: Enigo,
    held_modifiers: HashSet<&'static str>,
    held_keys: HashSet<&'static str>,
    held_buttons: HashSet<&'static str>,
}

impl InputController {
    pub fn new() -> Result<Self, InputError> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| InputError::Initialization(e.to_string()))?;
        Ok(Self { enigo, held_modifiers: HashSet::new(), held_keys: HashSet::new(), held_buttons: HashSet::new() })
    }

    pub fn move_mouse(&mut self, dx: i32, dy: i32) -> Result<(), InputError> {
        self.enigo.move_mouse(dx.clamp(-500, 500), dy.clamp(-500, 500), Coordinate::Rel).map_err(op)
    }

    pub fn mouse_button(&mut self, button: MouseButton, pressed: bool) -> Result<(), InputError> {
        let (key, name) = match button { MouseButton::Left => (Button::Left, "left"), MouseButton::Right => (Button::Right, "right"), MouseButton::Middle => (Button::Middle, "middle") };
        self.enigo.button(key, if pressed { Direction::Press } else { Direction::Release }).map_err(op)?;
        if pressed { self.held_buttons.insert(name); } else { self.held_buttons.remove(name); }
        Ok(())
    }

    pub fn scroll(&mut self, delta: i32) -> Result<(), InputError> {
        self.enigo.scroll(delta.clamp(-12, 12), Axis::Vertical).map_err(op)
    }

    pub fn text(&mut self, text: &str) -> Result<(), InputError> {
        if text.chars().count() > 1000 { return Err(InputError::Operation("text payload too long".into())); }
        self.enigo.text(text).map_err(op)
    }

    pub fn key(&mut self, remote: RemoteKey, pressed: bool, modifiers: &[Modifier]) -> Result<(), InputError> {
        let dir = if pressed { Direction::Press } else { Direction::Release };
        if pressed { for modifier in modifiers { self.set_modifier(*modifier, true)?; } }
        let (key, name) = map_key(remote);
        self.enigo.key(key, dir).map_err(op)?;
        if pressed { self.held_keys.insert(name); } else {
            self.held_keys.remove(name);
            for modifier in modifiers.iter().rev() { self.set_modifier(*modifier, false)?; }
        }
        Ok(())
    }

    pub fn media(&mut self, action: MediaAction) -> Result<(), InputError> {
        let key = match action {
            MediaAction::VolumeUp => Key::VolumeUp,
            MediaAction::VolumeDown => Key::VolumeDown,
            MediaAction::Mute => Key::VolumeMute,
            MediaAction::PlayPause => Key::MediaPlayPause,
            MediaAction::NextTrack => Key::MediaNextTrack,
            MediaAction::PreviousTrack => Key::MediaPrevTrack,
        };
        self.enigo.key(key, Direction::Click).map_err(op)
    }

    pub fn release_all(&mut self) {
        for key in [Key::Return, Key::Escape, Key::Backspace, Key::Tab, Key::UpArrow, Key::DownArrow, Key::LeftArrow, Key::RightArrow] { let _ = self.enigo.key(key, Direction::Release); }
        for key in [Key::Control, Key::Alt, Key::Shift, Key::Meta] { let _ = self.enigo.key(key, Direction::Release); }
        for button in [Button::Left, Button::Right, Button::Middle] { let _ = self.enigo.button(button, Direction::Release); }
        self.held_modifiers.clear(); self.held_keys.clear(); self.held_buttons.clear();
    }

    fn set_modifier(&mut self, modifier: Modifier, pressed: bool) -> Result<(), InputError> {
        let (key, name) = match modifier { Modifier::Ctrl => (Key::Control, "ctrl"), Modifier::Alt => (Key::Alt, "alt"), Modifier::Shift => (Key::Shift, "shift"), Modifier::Meta => (Key::Meta, "meta") };
        self.enigo.key(key, if pressed { Direction::Press } else { Direction::Release }).map_err(op)?;
        if pressed { self.held_modifiers.insert(name); } else { self.held_modifiers.remove(name); }
        Ok(())
    }
}

impl Drop for InputController { fn drop(&mut self) { self.release_all(); } }

fn map_key(key: RemoteKey) -> (Key, &'static str) {
    match key { RemoteKey::Enter => (Key::Return, "enter"), RemoteKey::Escape => (Key::Escape, "escape"), RemoteKey::Backspace => (Key::Backspace, "backspace"), RemoteKey::Tab => (Key::Tab, "tab"), RemoteKey::ArrowUp => (Key::UpArrow, "up"), RemoteKey::ArrowDown => (Key::DownArrow, "down"), RemoteKey::ArrowLeft => (Key::LeftArrow, "left"), RemoteKey::ArrowRight => (Key::RightArrow, "right") }
}
fn op(error: enigo::InputError) -> InputError { InputError::Operation(error.to_string()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_mapping_is_complete() { assert!(matches!(map_key(RemoteKey::Enter).0, Key::Return)); }
}
