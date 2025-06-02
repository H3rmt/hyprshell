use crate::config::Config;
use anyhow::bail;

pub fn check(config: &Config) -> anyhow::Result<()> {
    if config
        .windows
        .as_ref()
        .map(|w| w.scale >= 15f64 || w.scale <= 0f64)
        .unwrap_or(false)
    {
        bail!("Scale factor must be less than 15 and greater than 0");
    }

    if let Some(l) = &config.launcher {
        let mut used: Vec<char> = vec![];
        for engine in l
            .plugins
            .websearch
            .as_ref()
            .map(|ws| &ws.engines)
            .unwrap_or(&vec![])
        {
            if engine.url.is_empty() {
                bail!("Search engine url cannot be empty");
            }
            if engine.name.is_empty() {
                bail!("Search engine name cannot be empty");
            }
            if used.contains(&engine.key) {
                bail!("Duplicate search engine key: {}", engine.key);
            } else {
                used.push(engine.key);
            }
        }
        if l.plugins.calc.is_some() {
            #[cfg(not(feature = "calc"))]
            {
                bail!("Calc Plugin enabled but not compiled in, please enable the calc feature");
            }
        }
    };

    Ok(())
}
