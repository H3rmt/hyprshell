use super::*;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::default_instance;
use crate::instance::Instance;
use futures_lite::{Stream, StreamExt};

/// Event listener, but [Stream]
/// This is the new prefered way of listening for events
/// as its more idiomatic, and allows for more efficient memory management
///
/// # Examples
/// ```rust, no_run
/// use hyprland::prelude::*;
/// use hyprland::event_listener::EventStream;
/// use hyprland::Result as HResult;
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> HResult<()> {
///     use futures_lite::StreamExt;
///     use hyprland::instance::Instance;
///     let mut stream = EventStream::new();
///     while let Some(Ok(event)) = stream.next().await {
///          println!("{event:?}");
///     }
///     Ok(())
/// }
/// ```
#[must_use = "streams nothing unless polled"]
pub struct EventStream {
    stream: Pin<Box<dyn Stream<Item = crate::Result<Event>> + Send>>,
}
impl Default for EventStream {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStream {
    /// Creates a new [EventStream]
    pub fn new() -> Self {
        use crate::async_import::*;
        let stream = async_stream::try_stream! {
        let mut stream: UnixStream = default_instance()?.get_event_stream_async().await?;
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
                    for primed_event in event_primer_noexec(event, &mut active_windows)? {
                        yield primed_event;
                    }
                }
            }
        };
        Self {
            stream: Box::pin(stream),
        }
    }

    /// Creates a new [EventStream]
    pub fn instance_new(instance: Instance) -> Self {
        use crate::async_import::*;
        let stream = async_stream::try_stream! {
        let mut stream: UnixStream = instance.get_event_stream_async().await?;
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
                    for primed_event in event_primer_noexec(event, &mut active_windows)? {
                        yield primed_event;
                    }
                }
            }
        };
        Self {
            stream: Box::pin(stream),
        }
    }
}

impl Stream for EventStream {
    type Item = crate::Result<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().stream.poll_next(cx)
    }
}
