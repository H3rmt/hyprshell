/// Demostrats using hyprland-rs to listen for events
/// Usage: cargo run --example events
use hyprland::event_listener::EventListener;

fn main() -> hyprland::Result<()> {
    // Create a event listener
    let mut event_listener = EventListener::new();

    event_listener.add_active_window_changed_handler(|data| println!("{data:#?}"));
    event_listener.add_fullscreen_state_changed_handler(|fstate| {
        println!("Window {} fullscreen", if fstate { "is" } else { "is not" })
    });
    event_listener
        .add_active_monitor_changed_handler(|state| println!("Monitor state: {state:#?}"));
    event_listener.add_workspace_changed_handler(|id| println!("workspace changed to {id:?}"));

    // and execute the function
    // here we are using the blocking variant
    // but there is a async version too
    event_listener.start_listener()?;
    Ok(())
}
