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
///     let mut listener = event_listener::EventListener::new();
///     listener.add_workspace_changed_handler(|data| println!("{:#?}", data));
///     listener.start_listener_async().await?;
///     Ok(())
/// }
/// ```
pub struct EventListener {
    pub(crate) events: Events,
}

impl Default for EventListener {
    fn default() -> Self {
        Self::new()
    }
}

impl EventListener {
    /// This method creates a new EventListener instance
    ///
    /// ```rust
    /// use hyprland::event_listener;
    /// let mut listener = event_listener::EventListener::new();
    /// ```
    pub fn new() -> EventListener {
        EventListener {
            events: create_events(),
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
    ///     let mut listener = event_listener::EventListener::new();
    ///     listener.add_workspace_changed_handler(|data| println!("{:#?}", data));
    ///     listener.start_listener_async().await?;
    ///     Ok(())
    /// }
    /// ```
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
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
    ///     let mut listener = event_listener::EventListener::new();
    ///     listener.add_workspace_changed_handler(|data| println!("{:#?}", data));
    ///     let instance = default_instance()?;
    ///     listener.instance_start_listener_async(instance).await?;
    ///     Ok(())
    /// }
    /// ```
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
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
                self.event_primer(event, &mut active_windows)?;
            }
        }
        Ok(())
    }

    /// This method starts the event listener (blocking)
    ///
    /// This should be ran after all of your handlers are defined
    /// ```rust, no_run
    /// # use hyprland::{default_instance_panic, event_listener};
    /// # use hyprland_macros::async_closure;
    /// fn function() -> hyprland::Result<()> {
    ///     let mut listener = event_listener::EventListener::new();
    ///     listener.add_workspace_changed_handler(|data| println!("{:#?}", data));
    ///     listener.start_listener()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn start_listener(&mut self) -> crate::Result<()> {
        self.instance_start_listener(default_instance()?)
    }

    /// This method starts the event listener (blocking)
    ///
    /// This should be ran after all of your handlers are defined
    /// ```rust, no_run
    /// # use hyprland::{default_instance, event_listener};
    /// # use hyprland_macros::async_closure;
    /// fn function() -> hyprland::Result<()> {
    ///     let mut listener = event_listener::EventListener::new();
    ///     listener.add_workspace_changed_handler(|data| println!("{:#?}", data));
    ///     let instance = default_instance()?;
    ///     listener.instance_start_listener(instance)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn instance_start_listener(&mut self, instance: &Instance) -> crate::Result<()> {
        // use io::prelude::*;
        use std::io::Read;
        let mut stream = instance.get_event_stream()?;

        let mut active_windows = vec![];
        loop {
            let mut buffer = [0; 4096];
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read == 0 {
                // If no bytes were read, we can assume the stream is closed
                break;
            }
            let buf = &buffer[..bytes_read];
            let string = String::from_utf8_lossy(buf);
            let parsed: Vec<Event> = event_parser(&string)?;
            for event in parsed {
                self.event_primer(event, &mut active_windows)?;
            }
        }
        Ok(())
    }
}
