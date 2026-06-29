/// A monitor on which to apply a wallpaper, see [`crate::hyprpaper::Wallpaper`].
pub enum Monitor {
    /// A monitor port, such as "DP-1".
    Port(String),
    /// A monitor description, such as "Dell Inc. DELL P2419HC GNJJJ73".
    Description(String),
}

impl std::fmt::Display for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Port(port) => write!(f, "{port}"),
            Self::Description(description) => write!(f, "desc:{description}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check(monitor: Monitor, expected: &str) {
        assert_eq!(monitor.to_string(), expected);
    }

    #[test]
    fn test_port() {
        let monitor = Monitor::Port("DP-1".into());
        check(monitor, "DP-1");
    }

    #[test]
    fn test_description() {
        let monitor = Monitor::Description("Dell Inc. DELL P2419HC GNJJJ73".into());
        check(monitor, "desc:Dell Inc. DELL P2419HC GNJJJ73");
    }
}
