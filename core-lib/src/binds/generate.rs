use crate::binds::structs::ExecBind;
use anyhow::bail;
use config_lib::Modifier;

pub fn generate_bind_kill(kill_bind: &str) -> anyhow::Result<ExecBind> {
    let a = kill_bind
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let mods = match a.first() {
        Some(s) => {
            let mut parsed_mods = Vec::new();
            for str in s.split('+') {
                parsed_mods.push(Modifier::try_from(str)?.to_str());
            }
            parsed_mods
        }
        None => bail!("No mods specified"),
    };
    let key = match a.get(1) {
        Some(s) => Box::from(*s),
        None => bail!("No key provided in bind: {}", kill_bind),
    };

    let bind = ExecBind {
        key,
        mods,
        exec: format!("kill {}", std::process::id()).into_boxed_str(),
    };
    Ok(bind)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_bind_kill() {
        let bind = generate_bind_kill("ctrl+shift,k").unwrap();
        assert_eq!(bind.mods, vec!["ctrl", "shift"]);
        assert_eq!(bind.key, "k".into());
    }

    #[test]
    fn test_generate_bind_kill_2() {
        let bind = generate_bind_kill("ctrl+shift, k").unwrap();
        assert_eq!(bind.mods, vec!["ctrl", "shift"]);
        assert_eq!(bind.key, "k".into());
    }

    #[test]
    #[should_panic]
    fn test_generate_bind_kill_malformed() {
        let bind = generate_bind_kill("ctrl+shift+k").unwrap();
        assert_eq!(bind.mods, vec!["ctrl", "shift"]);
        assert_eq!(bind.key, "k".into());
    }

    #[test]
    #[should_panic]
    fn test_generate_bind_kill_no_mods_panic() {
        generate_bind_kill("k").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_generate_bind_kill_no_key_panic() {
        generate_bind_kill("ctrl+shift").unwrap();
    }
}
