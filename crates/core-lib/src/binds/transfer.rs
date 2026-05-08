use crate::transfer::ExternalTransferType;
use std::env;

/// # Panics
/// if the current executable couldn't be found
#[must_use]
pub fn get_hyprshell_path() -> String {
    env::current_exe()
        .expect("Current executable not found")
        .display()
        .to_string()
        .replace("(deleted)", "")
}

/// # Panics
/// if the transfer could not be serialized into a string
#[must_use]
pub fn generate_transfer_socat(transfer: &ExternalTransferType) -> String {
    format!(
        r"{} socat '{}'",
        get_hyprshell_path(),
        generate_transfer(transfer)
    )
}

/// # Panics
/// if the transfer could not be serialized into a string
#[must_use]
pub fn generate_transfer(transfer: &ExternalTransferType) -> String {
    serde_json::to_string(transfer).expect("serialize transfer")
}
