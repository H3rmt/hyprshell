use super::*;
use crate::default_instance;

/// A helper struct that provides the current fullscreen state
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub struct FullscreenState(
    /// State
    pub bool,
);

impl HyprData for FullscreenState {
    fn get() -> crate::Result<Self> {
        Self::instance_get(default_instance()?)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_async() -> crate::Result<Self> {
        Self::instance_get_async(default_instance()?).await
    }
    fn instance_get(instance: &crate::instance::Instance) -> crate::Result<Self> {
        Ok(Self(Workspace::instance_get_active(instance)?.fullscreen))
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_async(instance: &crate::instance::Instance) -> crate::Result<Self> {
        Ok(Self(
            Workspace::instance_get_active_async(instance)
                .await?
                .fullscreen,
        ))
    }
}

impl FullscreenState {
    /// This method returns a bool of the current fullscreen state
    pub fn bool(self) -> bool {
        self.0
    }
}
