use super::*;
use crate::default_instance;
use crate::instance::Instance;

/// This struct is used for adding event handlers and executing them on events
/// # The Event Listener
///
/// This struct holds what you need to create a event listener
///
/// ## Usage
///
/// ```rust, no_run
/// # use hyprland::event_listener;
/// # use hyprland_macros::async_closure;
/// #[tokio::main(flavor = "current_thread")]
/// async fn function() -> hyprland::Result<()> {
///     let mut listener = event_listener::AsyncEventListener::new();
///     listener.add_workspace_changed_handler(async_closure! { |id| println!("workspace changed to {id:?}") });
///     listener.start_listener_async().await?;
///     Ok(())
/// }
/// ```
pub struct AsyncEventListener {
    pub(crate) events: AsyncEvents,
}

impl Default for AsyncEventListener {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncEventListener {
    /// This method creates a new EventListener instance
    ///
    /// ```rust
    /// use hyprland::event_listener;
    /// let mut listener = event_listener::AsyncEventListener::new();
    /// ```
    pub fn new() -> Self {
        Self {
            events: create_events_async(),
        }
    }

    /// This method starts the event listener (async)
    ///
    /// This should be ran after all of your handlers are defined
    /// ```rust, no_run
    /// # use hyprland::event_listener;
    /// # use hyprland_macros::async_closure;
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn function() -> hyprland::Result<()> {
    ///     let mut listener = event_listener::AsyncEventListener::new();
    ///     listener.add_workspace_changed_handler(async_closure! { |id| println!("workspace changed to {id:?}") });
    ///     listener.start_listener_async().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn start_listener_async(&mut self) -> crate::Result<()> {
        self.instance_start_listener_async(default_instance()?)
            .await
    }

    /// This method starts the event listener (async)
    ///
    /// This should be ran after all of your handlers are defined
    /// ```rust, no_run
    /// # use hyprland::{default_instance, event_listener};
    /// # use hyprland_macros::async_closure;
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn function() -> hyprland::Result<()> {
    ///     let mut listener = event_listener::AsyncEventListener::new();
    ///     listener.add_workspace_changed_handler(async_closure! { |id| println!("workspace changed to {id:?}") });
    ///     let instance = default_instance()?;
    ///     listener.instance_start_listener_async(instance).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn instance_start_listener_async(
        &mut self,
        instance: &Instance,
    ) -> crate::Result<()> {
        use crate::async_import::*;

        let mut stream = instance.get_event_stream_async().await?;
        let mut active_windows = vec![];
        loop {
            let mut buffer = [0; 4096];
            let bytes_read = stream.read(&mut buffer).await?;
            if bytes_read == 0 {
                // If no bytes were read, we can assume the stream is closed
                break;
            }
            let buf = &buffer[..bytes_read];
            let string = String::from_utf8_lossy(buf);
            let parsed: Vec<Event> = event_parser(&string)?;
            for event in parsed {
                self.event_primer_exec_async(event, &mut active_windows)
                    .await?;
            }
        }
        Ok(())
    }
}
