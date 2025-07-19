use crate::transfer::TransferType;
use std::env;

pub fn get_hyprshell_path() -> String {
    env::current_exe()
        .expect("Current executable not found")
        .display()
        .to_string()
        .replace("(deleted)", "")
}

pub fn generate_transfer_socat(transfer: &TransferType) -> String {
    format!(
        r#"{} socat '{}'"#,
        get_hyprshell_path(),
        serde_json::to_string(transfer).expect("serialize transfer")
    )
}
