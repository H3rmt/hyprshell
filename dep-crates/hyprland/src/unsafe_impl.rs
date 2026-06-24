//! This module provides unsafe impls for several types, mainly for FFI purposes. Do not use unless you know what you are doing.

/// unsafe implementations for event listener structs
#[cfg(feature = "listener")]
pub mod listeners {
    use crate::event_listener::*;

    unsafe impl Send for AsyncEventListener {}
    unsafe impl Sync for AsyncEventListener {}

    unsafe impl Send for EventListener {}
    unsafe impl Sync for EventListener {}

    unsafe impl Send for WindowMoveEvent {}
    unsafe impl Sync for WindowMoveEvent {}

    unsafe impl Send for WindowOpenEvent {}
    unsafe impl Sync for WindowOpenEvent {}

    unsafe impl Send for LayoutEvent {}
    unsafe impl Sync for LayoutEvent {}

    unsafe impl Send for State {}
    unsafe impl Sync for State {}

    unsafe impl Send for WindowEventData {}
    unsafe impl Sync for WindowEventData {}

    unsafe impl Send for MonitorEventData {}
    unsafe impl Sync for MonitorEventData {}

    unsafe impl Send for WindowFloatEventData {}
    unsafe impl Sync for WindowFloatEventData {}
}
