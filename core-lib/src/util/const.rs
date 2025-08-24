pub const APPLICATION_ID: &str = "com.github.h3rmt.hyprshell";
pub const APPLICATION_TEST_ID: &str = "com.github.h3rmt.hyprshell-test";
pub const OVERVIEW_NAMESPACE: &str = "hyprshell_overview";
pub const SWITCH_NAMESPACE: &str = "hyprshell_switch";
pub const LAUNCHER_NAMESPACE: &str = "hyprshell_launcher";

// from https://github.com/i3/i3/blob/next/i3-sensible-terminal
// shorted to only the most common ones that I know support -e option
pub const TERMINALS: [&str; 9] = [
    "alacritty",
    "kitty",
    "wezterm",
    "foot",
    "qterminal",
    "lilyterm",
    "tilix",
    "terminix",
    "konsole",
];
