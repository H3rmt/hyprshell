use tracing::{trace, warn};

pub fn test() -> anyhow::Result<()> {
    match hyprland_plugin::generate() {
        Ok(path) => {
            trace!("generated plugin at {:?}", path);
        }
        Err(err) => {
            warn!("unable to generate plugin: {err:?}")
        }
    }

    Ok(())
}
