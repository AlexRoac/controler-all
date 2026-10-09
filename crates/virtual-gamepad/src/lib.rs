use async_trait::async_trait;
use remotepad_protocol::GamepadState;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum GamepadError {
    #[error("invalid gamepad state")]
    InvalidState,
    #[error("virtual gamepad backend is unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait VirtualGamepadBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_real_xinput(&self) -> bool;
    async fn update(&self, state: &GamepadState) -> Result<(), GamepadError>;
    async fn reset(&self) -> Result<(), GamepadError>;
}

#[derive(Default)]
pub struct DiagnosticGamepad {
    state: RwLock<GamepadState>,
}

impl DiagnosticGamepad {
    pub fn new() -> Arc<Self> { Arc::new(Self::default()) }
    pub async fn snapshot(&self) -> GamepadState { self.state.read().await.clone() }
}

#[async_trait]
impl VirtualGamepadBackend for DiagnosticGamepad {
    fn name(&self) -> &'static str { "diagnostic" }
    fn is_real_xinput(&self) -> bool { false }
    async fn update(&self, state: &GamepadState) -> Result<(), GamepadError> {
        if !state.is_valid() { return Err(GamepadError::InvalidState); }
        *self.state.write().await = state.clone();
        Ok(())
    }
    async fn reset(&self) -> Result<(), GamepadError> {
        *self.state.write().await = GamepadState::default();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disconnect_reset_releases_all_controls() {
        let backend = DiagnosticGamepad::default();
        let mut active = GamepadState::default();
        active.buttons.a = true;
        active.left_stick.x = 0.8;
        backend.update(&active).await.unwrap();
        backend.reset().await.unwrap();
        assert_eq!(backend.snapshot().await, GamepadState::default());
    }

    #[tokio::test]
    async fn preserves_simultaneous_and_held_buttons_until_release() {
        let backend = DiagnosticGamepad::default();
        let mut active = GamepadState::default();
        active.buttons.a = true;
        active.buttons.rb = true;
        backend.update(&active).await.unwrap();
        let held = backend.snapshot().await;
        assert!(held.buttons.a && held.buttons.rb);
        active.buttons.a = false;
        backend.update(&active).await.unwrap();
        let released = backend.snapshot().await;
        assert!(!released.buttons.a && released.buttons.rb);
    }
}

