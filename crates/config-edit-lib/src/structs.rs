use adw::gtk::{Button, CheckButton, Grid, SpinButton};

pub struct GTKConfig {
    pub windows: GTKWindows,
    pub save: Button,
}

pub struct GTKWindows {
    pub enabled: CheckButton,
    pub view: Grid,
    pub scale: SpinButton,
    pub items_per_row: SpinButton,
}
