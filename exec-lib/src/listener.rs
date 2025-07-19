use core_lib::WarnWithDetails;
use hyprland::default_instance_panic;
use tracing::info;

pub async fn monitor_listener<F>(callback: F)
where
    F: Fn(&'static str) + 'static + Clone,
{
    let mut event_listener = hyprland::event_listener::EventListener::new();
    let callback_clone = callback.clone();
    event_listener.add_monitor_added_handler(move |_data| {
        callback("monitor added");
    });
    event_listener.add_monitor_removed_handler(move |_data| {
        callback_clone("monitor removed");
    });
    info!("Starting monitor added/removed listener");
    event_listener
        .start_listener_async(default_instance_panic())
        .await
        .warn_details("Failed to start monitor added/removed listener");
}

pub async fn hyprland_config_listener<F>(callback: F)
where
    F: Fn(&'static str) + 'static,
{
    let mut event_listener = hyprland::event_listener::EventListener::new();
    event_listener.add_config_reloaded_handler(move || {
        callback("hyprland config reload");
    });
    info!("Starting hyprland config reload listener");
    event_listener
        .start_listener_async(default_instance_panic())
        .await
        .warn_details("Failed to start hyprland config reload listener");
}
