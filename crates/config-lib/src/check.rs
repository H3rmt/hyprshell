use crate::Config;
use anyhow::bail;

pub fn check(config: &Config) -> anyhow::Result<()> {
    if config
        .windows
        .as_ref()
        .is_some_and(|w| w.scale >= 15f64 || w.scale <= 0f64)
    {
        bail!("Scale factor must be less than 15 and greater than 0");
    }

    if config
        .windows
        .as_ref()
        .and_then(|w| w.overview.as_ref())
        .is_some_and(|o| o.launcher.launch_modifier == o.modifier)
    {
        bail!(
            "Launcher modifier cannot be the same as overview open modifier. (pressing the modifier will just close the overview instead of launching an app)"
        );
    }

    if config
        .windows
        .as_ref()
        .and_then(|w| w.overview.as_ref())
        .is_some_and(|o| matches!(&*o.key, "super" | "alt" | "control" | "ctrl"))
    {
        bail!(
            "If a modifier key is used to open it must include _l or _r at the end. (e.g. super_l, alt_r, etc)\nctrl_l / _r is NOT a valid modifier key, only control_l / _r is"
        );
    }

    if let Some(l) = &config
        .windows
        .as_ref()
        .and_then(|w| w.overview.as_ref().map(|o| &o.launcher))
    {
        if let Some(dt) = &l.default_terminal {
            if dt.is_empty() {
                bail!("Default terminal command cannot be empty");
            }
        }

        let mut used: Vec<char> = vec![];
        for engine in l
            .plugins
            .websearch
            .as_ref()
            .map_or(&vec![], |ws| &ws.engines)
        {
            if engine.url.is_empty() {
                bail!("Search engine url cannot be empty");
            }
            if engine.name.is_empty() {
                bail!("Search engine name cannot be empty");
            }
            if used.contains(&engine.key) {
                bail!("Duplicate search engine key: {}", engine.key);
            }
            used.push(engine.key);
        }
        if l.plugins.calc.is_some() {
            #[cfg(not(feature = "launcher_calc_plugin"))]
            {
                bail!("Calc Plugin enabled but not compiled in, please enable the calc feature");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::*;

    fn full() -> Config {
        Config {
            windows: Some(Windows {
                overview: Some(Overview::default()),
                switch: Some(Switch::default()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_valid_config() {
        let config = full();
        assert!(check(&config).is_ok());
    }

    #[test]
    fn test_invalid_scale() {
        let mut config = full();
        config
            .windows
            .as_mut()
            .expect("config option missing")
            .scale = 20.0;
        assert!(check(&config).is_err());
        config
            .windows
            .as_mut()
            .expect("config option missing")
            .scale = 0.0;
        assert!(check(&config).is_err());
    }

    #[test]
    fn test_same_modifier() {
        let mut config = full();
        let overview = config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview
            .as_mut()
            .expect("config option missing");
        overview.launcher.launch_modifier = overview.modifier;
        assert!(check(&config).is_err());
    }

    #[test]
    fn test_invalid_key() {
        let mut config = full();
        let overview = config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview
            .as_mut()
            .expect("config option missing");
        overview.key = Box::from("super");
        assert!(check(&config).is_err());
    }

    #[test]
    fn test_duplicate_engine_key() {
        let mut config = full();
        let launcher = &mut config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview
            .as_mut()
            .expect("config option missing")
            .launcher;
        if let Some(ws) = launcher.plugins.websearch.as_mut() {
            ws.engines.push(SearchEngine {
                key: 'g',
                name: Box::from("Google2"),
                url: Box::from("https://google2.com"),
            });
        }
        assert!(check(&config).is_err());
    }

    #[test]
    fn test_empty_engine_url() {
        let mut config = full();
        let launcher = &mut config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview
            .as_mut()
            .expect("config option missing")
            .launcher;
        if let Some(ws) = launcher.plugins.websearch.as_mut() {
            ws.engines[0].url = Box::from("");
        }
        assert!(check(&config).is_err());
    }

    #[test]
    fn test_empty_engine_name() {
        let mut config = full();
        let launcher = &mut config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview
            .as_mut()
            .expect("config option missing")
            .launcher;
        if let Some(ws) = launcher.plugins.websearch.as_mut() {
            ws.engines[0].name = Box::from("");
        }
        assert!(check(&config).is_err());
    }

    #[test]
    fn test_empty_terminal() {
        let mut config = full();
        let launcher = &mut config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview
            .as_mut()
            .expect("config option missing")
            .launcher;
        launcher.default_terminal = Some(Box::from(""));
        assert!(check(&config).is_err());
    }
}
