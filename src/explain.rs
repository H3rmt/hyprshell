use config_lib::{explain, load_and_migrate_config};
use core_lib::util::daemon_running;
use std::path::Path;

#[allow(clippy::print_stderr, clippy::print_stdout)]
pub fn explain_config(config_file: &Path, add_how_to_explain_again: bool) {
    let config = match load_and_migrate_config(config_file, true) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "\x1b[1m\x1b[31mConfig is invalid ({}):\x1b[0m {err:?}\n",
                config_file.display()
            );
            return;
        }
    };
    let info = explain(&config, Some(config_file), true);
    println!("{info}");

    if daemon_running() {
        println!("Daemon \x1b[32mrunning\x1b[0m");
    } else {
        eprintln!(
            "Daemon \x1b[31mnot running\x1b[0m, start it with `hyprshell run` or `systemctl --user enable --now hyprshell`"
        );
    }

    if add_how_to_explain_again {
        println!("\nTo explain the config again, run `hyprshell explain`\n");
    }
}
