use hyprland::dispatch::DispatchType;
use hyprland::instance::Instance;
use hyprland::keyword::Keyword;
use hyprland::{default_instance_panic, dispatch};
/// Demonstates the use of Hyprland-rs for asyncronous creation key bindings
/// and using submaps
///
/// Usage: cargo run --example bind
use std::io::Read;

#[tokio::main(flavor = "current_thread")]
async fn main() -> hyprland::Result<()> {
    let instance1 = default_instance_panic();
    Keyword::instance_set_async(instance1, "submap", "example").await?;
    hyprland::bind!(async; instance1, SUPER, Key, "I" => ToggleFloating, None).await?;
    hyprland::bind!(async; instance1, l | CTRL ALT, Key, "Delete" => Exec, "sudo reboot").await?; // Reboot including from lock screen
    hyprland::bind!(async; instance1, e | SUPER, Key, "C" => KillActiveWindow).await?; // Kill all your windows
    Keyword::instance_set_async(instance1, "submap", "reset").await?;

    let instance2 = Instance::from_instance("long instance name".to_string())?;
    Keyword::instance_set_async(&instance2, "submap", "example2").await?;
    hyprland::bind!(async; &instance2, SUPER, Key, "I" => ToggleFloating, None).await?;
    hyprland::bind!(async; &instance2, l | CTRL ALT, Key, "Delete" => Exec, "sudo reboot").await?; // Reboot including from lock screen
    hyprland::bind!(async; &instance2, e | SUPER, Key, "C" => KillActiveWindow).await?; // Kill all your windows
    Keyword::instance_set_async(&instance2, "submap", "reset2").await?;

    Keyword::set_async("submap", "example").await?;
    hyprland::bind!(async; SUPER, Key, "I" => ToggleFloating, None).await?;
    hyprland::bind!(async; l | CTRL ALT, Key, "Delete" => Exec, "sudo reboot").await?; // Reboot including from lock screen
    hyprland::bind!(async; e | SUPER, Key, "C" => KillActiveWindow).await?; // Kill all your windows
    Keyword::set_async("submap", "reset").await?;

    let instance3 = Instance::from_current_env()?;
    dispatch!(async; &instance3; Custom, "submap", "example").await?;
    println!("Press enter to revert to default keymap");
    let _ = std::io::stdin().read(&mut [0u8])?;
    dispatch!(async; &instance3; Custom, "submap", "reset").await?;
    Ok(())
}
