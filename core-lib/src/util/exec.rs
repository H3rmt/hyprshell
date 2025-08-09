#[derive(Debug, Clone)]
pub enum ExecType {
    Flatpak(Box<str>, Box<str>),
    PWA(Box<str>, Box<str>),
    FlatpakPWA(Box<str>, Box<str>),
    Absolute(Box<str>, Box<str>),
    AppImage(Box<str>, Box<str>),
    Relative(Box<str>),
}

const UNKNOWN_EXEC: &str = "unknown";

pub fn analyse_exec(exec: &str) -> ExecType {
    let exec_trim = exec.replace(['\'', '"'], "");
    // pwa detection
    if exec.contains("--app-id=") && exec.contains("--profile-directory=") {
        // "flatpak 'run'" = pwa from browser inside flatpak
        if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
            let browser_exec_in_flatpak = exec_trim
                .split_whitespace()
                .find(|s| s.contains("--command="))
                .and_then(|s| {
                    s.split('=')
                        .next_back()
                        .and_then(|s| s.split('/').next_back())
                })
                .unwrap_or(UNKNOWN_EXEC);
            let flatpak_identifier = exec_trim
                .split_whitespace()
                .skip(2)
                .find(|arg| !arg.starts_with("--"))
                .unwrap_or(UNKNOWN_EXEC);
            ExecType::FlatpakPWA(
                Box::from(flatpak_identifier),
                Box::from(browser_exec_in_flatpak),
            )
        } else {
            // normal PWA
            let browser_exec = exec
                .split_whitespace()
                .next()
                .and_then(|s| s.split('/').next_back())
                .unwrap_or(UNKNOWN_EXEC);
            let browser_full_exec = exec.split_whitespace().next().unwrap_or(UNKNOWN_EXEC);
            ExecType::PWA(Box::from(browser_exec), Box::from(browser_full_exec))
        }
        // flatpak detection
    } else if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
        let command_in_flatpak = exec_trim
            .split_whitespace()
            .find(|s| s.contains("--command="))
            .and_then(|s| {
                s.split('=')
                    .next_back()
                    .and_then(|s| s.split('/').next_back())
            })
            .unwrap_or(UNKNOWN_EXEC);
        let flatpak_identifier = exec_trim
            .split_whitespace()
            .skip(2)
            .find(|arg| !arg.starts_with("--"))
            .unwrap_or(UNKNOWN_EXEC);
        ExecType::Flatpak(Box::from(flatpak_identifier), Box::from(command_in_flatpak))
    } else if exec_trim.contains(".AppImage") {
        // AppImage detection
        let appimage_name = exec_trim
            .split_whitespace()
            .next()
            .and_then(|s| s.split('/').next_back())
            .and_then(|s| s.split('_').next())
            .unwrap_or(UNKNOWN_EXEC);
        ExecType::AppImage(Box::from(appimage_name), Box::from(exec))
    } else if exec_trim.starts_with('/') {
        let exec_name = exec_trim
            .split_whitespace()
            .next()
            .and_then(|s| s.split('/').next_back())
            .unwrap_or(UNKNOWN_EXEC);
        ExecType::Absolute(Box::from(exec_name), Box::from(exec))
    } else {
        ExecType::Relative(Box::from(exec_trim))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_exec() {
        assert!(matches!(
            analyse_exec("nautilus --new-window"),
            ExecType::Relative(ref s) if &**s == "nautilus --new-window"
        ));
    }

    #[test]
    fn test_flatpak_pwa_exec() {
        assert!(matches!(
            analyse_exec(
                "flatpak 'run' '--command=/app/bin/chromium' 'org.chromium.Chromium' '--profile-directory=Default' '--app-id=awf'"
            ),
            ExecType::FlatpakPWA(ref id, ref browser) if &**id == "org.chromium.Chromium" && &**browser == "chromium"
        ));
    }

    #[test]
    fn test_appimage_exec() {
        assert!(matches!(
            analyse_exec(
                "/home/user/Applications/ungoogled-chromium_71.0.3578.98-2_linux_awf.AppImage %u"
            ),
            ExecType::AppImage(ref name, ref path) if &**name == "ungoogled-chromium" && &**path == "/home/user/Applications/ungoogled-chromium_71.0.3578.98-2_linux_awf.AppImage %u"
        ));
    }

    #[test]
    fn test_absolute_pwa_exec() {
        assert!(matches!(
            analyse_exec(
                "/opt/google/chrome/google-chrome --profile-directory=Default --app-id=awf"
            ),
            ExecType::PWA(ref browser, ref path) if &**browser == "google-chrome" && &**path == "/opt/google/chrome/google-chrome"
        ));
    }

    #[test]
    fn test_flatpak_exec() {
        assert!(matches!(
            analyse_exec("flatpak run org.mozilla.firefox"),
            ExecType::Flatpak(ref id, ref command) if &**id == "org.mozilla.firefox" && &**command == "unknown"
        ));
    }

    #[test]
    fn test_absolute_exec() {
        assert!(matches!(
            analyse_exec("/usr/bin/firefox"),
            ExecType::Absolute(ref name, ref path) if &**name == "firefox" && &**path == "/usr/bin/firefox"
        ));
    }
}
