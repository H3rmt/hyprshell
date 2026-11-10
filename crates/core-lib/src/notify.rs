use notify_rust::{Hint, Notification};
use std::time::Duration;
use tracing::{info, warn};

pub fn notify(body: &str, duration: Duration) {
    info!("{}", body);
    let _ = Notification::new()
        .summary("Hyprshell")
        .body(body)
        .appname("hyprshell-settings")
        .timeout(duration)
        .urgency(notify_rust::Urgency::Normal)
        .show();
}

pub fn notify_resident(body: &str, duration: Duration) {
    info!("{}", body);
    let _ = Notification::new()
        .summary("Hyprshell")
        .body(body)
        .appname("hyprshell")
        .timeout(duration)
        .hint(Hint::Resident(true))
        .timeout(Duration::from_secs(0))
        .urgency(notify_rust::Urgency::Normal)
        .show();
}

pub fn notify_warn(body: &str) {
    warn!("{}", body);
    let _ = Notification::new()
        .summary("Hyprshell")
        .body(body)
        .appname("hyprshell")
        .timeout(Duration::from_secs(8))
        .urgency(notify_rust::Urgency::Critical)
        .show();
}
