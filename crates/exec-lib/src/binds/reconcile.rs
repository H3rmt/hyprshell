use super::{apply_exec_bind_legacy, lua_binding};
use anyhow::{Context, bail, ensure};
use core_lib::binds::ExecBind;
use hyprland::data::Binds;
use hyprland::keyword::Keyword;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const REGISTRY: &str = include_str!("registry.lua");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Spec {
    modmask: u16,
    key: String,
    exec: String,
    release: bool,
    desc: String,
}

impl Spec {
    fn new(binding: &ExecBind) -> anyhow::Result<Self> {
        ensure!(!binding.key.is_empty(), "Binding key cannot be empty");
        ensure!(
            !binding.key.chars().any(|character| character.is_control()
                || matches!(character, ',' | '"' | '\\' | '+' | ' ')),
            "Invalid binding key: {}",
            binding.key
        );
        let mut modmask = 0;
        for modifier in &binding.mods {
            modmask |= match modifier.to_ascii_lowercase().as_str() {
                "shift" => 1,
                "ctrl" | "control" => 4,
                "alt" => 8,
                "super" | "win" => 64,
                "none" => 0,
                _ => bail!("Unknown binding modifier: {modifier}"),
            };
        }
        Ok(Self {
            modmask,
            key: binding.key.to_string(),
            exec: binding.exec.clone(),
            release: binding.release,
            desc: binding.desc.clone(),
        })
    }

    fn chord(&self) -> String {
        format!("{}:{}", self.modmask, self.key.to_ascii_lowercase())
    }

    fn modifiers(&self) -> Vec<&'static str> {
        [(1, "SHIFT"), (4, "CTRL"), (8, "ALT"), (64, "SUPER")]
            .into_iter()
            .filter_map(|(mask, name)| (self.modmask & mask != 0).then_some(name))
            .collect()
    }

    fn binding(&self) -> ExecBind {
        ExecBind {
            mods: self.modifiers(),
            key: self.key.clone().into(),
            exec: self.exec.clone(),
            release: self.release,
            desc: self.desc.clone(),
        }
    }
}

fn live_chord(binding: &Value) -> anyhow::Result<String> {
    let mask = binding["modmask"]
        .as_u64()
        .context("Missing binding modmask")?;
    let keycode = binding["keycode"]
        .as_i64()
        .context("Missing binding keycode")?;
    let key = if keycode > 0 {
        format!("code:{keycode}")
    } else {
        binding["key"]
            .as_str()
            .context("Missing binding key")?
            .to_ascii_lowercase()
    };
    Ok(format!("{mask}:{key}"))
}

trait Transport {
    fn snapshot(&mut self) -> anyhow::Result<Vec<Value>>;
    fn lua(&mut self, code: &str) -> anyhow::Result<()>;
    fn bind(&mut self, spec: &Spec) -> anyhow::Result<()>;
    fn unbind(&mut self, spec: &Spec) -> anyhow::Result<()>;
}

struct HyprlandTransport;

impl Transport for HyprlandTransport {
    fn snapshot(&mut self) -> anyhow::Result<Vec<Value>> {
        Binds::get_raw().context("Unable to read the complete Hyprland binding table")
    }

    fn lua(&mut self, code: &str) -> anyhow::Result<()> {
        hyprland::EvalRaw::new(code)
            .eval()
            .context("Hyprshell binding Lua request failed")
    }

    fn bind(&mut self, spec: &Spec) -> anyhow::Result<()> {
        apply_exec_bind_legacy(&spec.binding())
    }

    fn unbind(&mut self, spec: &Spec) -> anyhow::Result<()> {
        Keyword::set(
            "unbind",
            format!("{},{}", spec.modifiers().join(" "), spec.key),
        )
        .context("Unable to remove owned legacy binding")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Owned {
    spec: Spec,
    fingerprint: Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    version: u8,
    instance: String,
    entries: Vec<Owned>,
}

struct Store {
    path: PathBuf,
    instance: String,
    _lock: File,
}

impl Store {
    fn open(cache_dir: &Path, instance: &str) -> anyhow::Result<Self> {
        ensure!(
            !instance.is_empty()
                && instance
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric()
                        || matches!(character, '-' | '_')),
            "Invalid Hyprland instance signature"
        );
        let directory = cache_dir.join("keybinds-v1");
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&directory)?;
        let path = directory.join(format!("{instance}.json"));
        let lock_path = directory.join(format!("{instance}.lock"));
        for candidate in [&path, &lock_path] {
            if let Ok(metadata) = fs::symlink_metadata(candidate) {
                ensure!(
                    metadata.is_file() && !metadata.file_type().is_symlink(),
                    "Refusing unsafe binding ledger path: {}",
                    candidate.display()
                );
            }
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(lock_path)?;
        lock.lock()
            .context("Unable to lock the binding ownership ledger")?;
        Ok(Self {
            path,
            instance: instance.to_string(),
            _lock: lock,
        })
    }

    fn load(&self) -> anyhow::Result<Ledger> {
        let ledger = match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice::<Ledger>(&bytes)
                .context("Invalid binding ownership ledger; refusing to guess ownership")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ledger {
                version: 1,
                instance: self.instance.clone(),
                entries: Vec::new(),
            },
            Err(error) => return Err(error.into()),
        };
        ensure!(
            ledger.version == 1 && ledger.instance == self.instance,
            "Incompatible binding ownership ledger"
        );
        for entry in &ledger.entries {
            ensure!(
                entry.spec.chord() == live_chord(&entry.fingerprint)?,
                "Binding ownership ledger has an inconsistent chord"
            );
        }
        Ok(ledger)
    }

    fn save(&self, ledger: &Ledger) -> anyhow::Result<()> {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let temporary = self.path.with_extension(format!(
            "tmp-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)?;
            file.write_all(&serde_json::to_vec(ledger)?)?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            File::open(self.path.parent().context("Ledger directory is missing")?)?.sync_all()?;
            Ok::<_, anyhow::Error>(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result.context("Unable to persist binding ownership; no untracked bindings will be removed")
    }
}

fn lua_string(value: &str) -> String {
    let mut escaped = String::from("\"");
    for byte in value.bytes() {
        write!(escaped, "\\{byte:03}").expect("Writing into a String cannot fail");
    }
    escaped.push('"');
    escaped
}

fn lua_value(value: &Value) -> anyhow::Result<String> {
    match value {
        Value::Null => Ok("nil".into()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        Value::String(value) => Ok(lua_string(value)),
        Value::Array(values) => Ok(format!(
            "{{{}}}",
            values
                .iter()
                .map(lua_value)
                .collect::<anyhow::Result<Vec<_>>>()?
                .join(",")
        )),
        Value::Object(values) => Ok(format!(
            "{{{}}}",
            values
                .iter()
                .map(|(key, value)| Ok(format!("[{}]={}", lua_string(key), lua_value(value)?)))
                .collect::<anyhow::Result<Vec<_>>>()?
                .join(",")
        )),
    }
}

fn grouped(specs: &[Spec]) -> BTreeMap<String, Vec<&Spec>> {
    let mut groups = BTreeMap::<_, Vec<_>>::new();
    for spec in specs {
        groups.entry(spec.chord()).or_default().push(spec);
    }
    groups
}

fn reconcile_lua(transport: &mut impl Transport, specs: &[Spec]) -> anyhow::Result<()> {
    let live = transport.snapshot()?;
    for binding in &live {
        live_chord(binding)?;
        for field in ["dispatcher", "arg", "submap", "key"] {
            ensure!(binding[field].is_string(), "Missing binding field: {field}");
        }
    }
    let mut groups = Vec::new();
    for (chord, wanted) in grouped(specs) {
        let mut entries = Vec::new();
        for spec in wanted {
            entries.push(format!(
                "{{spec={},create=function() return hl.bind({}) end}}",
                lua_string(&serde_json::to_string(spec)?),
                lua_binding(&spec.binding())
            ));
        }
        groups.push(format!(
            "[{}]={{{}}}",
            lua_string(&chord),
            entries.join(",")
        ));
    }
    transport.lua(&format!(
        "local reconcile = (function()\n{REGISTRY}\nend)(); reconcile({{{}}}, {})",
        groups.join(","),
        lua_value(&Value::Array(live))?
    ))
}

fn all_owned<'records>(
    records: impl IntoIterator<Item = &'records Value>,
    entries: &[Owned],
) -> bool {
    let mut fingerprints: Vec<_> = entries.iter().map(|entry| &entry.fingerprint).collect();
    records.into_iter().all(|record| {
        fingerprints
            .iter()
            .position(|fingerprint| *fingerprint == record)
            .is_some_and(|index| {
                fingerprints.remove(index);
                true
            })
    })
}

fn register_legacy(
    transport: &mut impl Transport,
    spec: &Spec,
    ledger: &mut Ledger,
    store: &Store,
) -> anyhow::Result<()> {
    let chord = spec.chord();
    let before = transport.snapshot()?;
    let mut matching = Vec::new();
    for binding in &before {
        if live_chord(binding)? == chord {
            matching.push(binding);
        }
    }
    ensure!(
        all_owned(matching, &ledger.entries),
        "An unowned binding appeared at {chord}; refusing to register"
    );
    transport.bind(spec)?;
    let after = transport.snapshot()?;
    let added: Vec<_> = after
        .into_iter()
        .filter(|binding| !before.contains(binding))
        .collect();
    ensure!(
        added.len() == 1,
        "Unable to identify the newly registered binding; refusing to guess ownership"
    );
    let fingerprint = added
        .into_iter()
        .next()
        .context("Registered binding is missing")?;
    ensure!(
        live_chord(&fingerprint)? == chord
            && fingerprint["dispatcher"] == "exec"
            && fingerprint["arg"] == spec.exec
            && fingerprint["release"] == spec.release
            && fingerprint["submap"] == "",
        "Unexpected registered binding; refusing to record ownership"
    );
    ledger.entries.push(Owned {
        spec: spec.clone(),
        fingerprint,
    });
    store.save(ledger)
}

fn reconcile_legacy(
    transport: &mut impl Transport,
    specs: &[Spec],
    store: &Store,
) -> anyhow::Result<()> {
    let mut ledger = store.load()?;
    let live = transport.snapshot()?;
    ledger
        .entries
        .retain(|entry| live.contains(&entry.fingerprint));
    let desired = grouped(specs);
    let mut touched = Vec::new();
    for binding in &live {
        let chord = live_chord(binding)?;
        let has_owned = ledger
            .entries
            .iter()
            .any(|entry| entry.spec.chord() == chord);
        if desired.contains_key(&chord) || has_owned {
            touched.push(binding);
        }
    }
    ensure!(
        all_owned(touched.iter().copied(), &ledger.entries),
        "Hyprshell will not change unowned bindings at {} (including other submaps). Resolve the conflicts or start a fresh session manually.",
        touched
            .iter()
            .map(|binding| live_chord(binding))
            .collect::<anyhow::Result<Vec<_>>>()?
            .join(", ")
    );
    store.save(&ledger)?;
    let mut chords: Vec<_> = ledger
        .entries
        .iter()
        .map(|entry| entry.spec.chord())
        .chain(desired.keys().cloned())
        .collect();
    chords.sort();
    chords.dedup();
    for chord in chords {
        let existing: Vec<_> = ledger
            .entries
            .iter()
            .filter(|entry| entry.spec.chord() == chord)
            .map(|entry| &entry.spec)
            .collect();
        let wanted = desired.get(&chord).cloned().unwrap_or_default();
        if existing == wanted {
            continue;
        }
        let current = transport.snapshot()?;
        let matching: Vec<_> = current
            .iter()
            .filter_map(|binding| match live_chord(binding) {
                Ok(found) if found == chord => Some(Ok(binding)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<anyhow::Result<_>>()?;
        ensure!(
            all_owned(matching.iter().copied(), &ledger.entries),
            "Binding ownership changed before removing {chord}; refusing to unbind"
        );
        if let Some(previous) = existing.first() {
            if !matching.is_empty() {
                transport.unbind(previous)?;
            }
            ledger.entries.retain(|entry| entry.spec.chord() != chord);
            store.save(&ledger)?;
        }
        for spec in wanted {
            register_legacy(transport, spec, &mut ledger, store)?;
        }
    }
    Ok(())
}

fn reconcile(
    transport: &mut impl Transport,
    specs: &[Spec],
    cache_dir: &Path,
    instance: &str,
) -> anyhow::Result<()> {
    match transport
        .lua("assert(type(hl.bind) == 'function', 'Hyprland Lua binding API is unavailable')")
    {
        Ok(()) => reconcile_lua(transport, specs),
        Err(error)
            if format!("{error:#}")
                .contains("eval is only supported with the lua config manager") =>
        {
            let store = Store::open(cache_dir, instance)?;
            reconcile_legacy(transport, specs, &store)
        }
        Err(error) => {
            Err(error).context("Unable to select a binding backend; no bindings were changed")
        }
    }
}

pub fn reconcile_exec_binds(bindings: &[ExecBind], cache_dir: &Path) -> anyhow::Result<()> {
    let mut specs = bindings
        .iter()
        .map(Spec::new)
        .collect::<anyhow::Result<Vec<_>>>()?;
    specs.sort_by_cached_key(|spec| (spec.chord(), spec.release));
    for pair in specs.windows(2) {
        ensure!(
            pair[0].chord() != pair[1].chord() || pair[0].release != pair[1].release,
            "Duplicate Hyprshell binding chord: {}",
            pair[0].chord()
        );
    }
    let instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("Hyprland instance signature is missing")?;
    reconcile(&mut HyprlandTransport, &specs, cache_dir, &instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Default)]
    struct Mock {
        live: Vec<Value>,
        mutations: Vec<String>,
        lua_available: bool,
        fail_probe: bool,
        fail_bind: bool,
        fail_lua: bool,
    }

    fn fingerprint(spec: &Spec) -> Value {
        json!({"modmask":spec.modmask,"key":spec.key,"keycode":0,"dispatcher":"exec","arg":spec.exec,"release":spec.release,"repeat":!spec.release,"submap":"","description":"","transparent":spec.release,"custom_future_flag":true})
    }

    impl Transport for Mock {
        fn snapshot(&mut self) -> anyhow::Result<Vec<Value>> {
            Ok(self.live.clone())
        }

        fn lua(&mut self, code: &str) -> anyhow::Result<()> {
            if self.fail_probe {
                bail!("Disconnected compositor");
            }
            if !self.lua_available {
                bail!("eval is only supported with the lua config manager");
            }
            if code.starts_with("assert(") {
                return Ok(());
            }
            self.mutations.push("lua".into());
            ensure!(
                !self.fail_lua,
                "Lua registration failed after backend selection"
            );
            Ok(())
        }

        fn bind(&mut self, spec: &Spec) -> anyhow::Result<()> {
            ensure!(!self.fail_bind, "Simulated bind failure");
            self.mutations.push(format!("bind:{}", spec.chord()));
            self.live.push(fingerprint(spec));
            Ok(())
        }

        fn unbind(&mut self, spec: &Spec) -> anyhow::Result<()> {
            self.mutations.push(format!("unbind:{}", spec.chord()));
            self.live
                .retain(|binding| live_chord(binding).expect("mock chord") != spec.chord());
            Ok(())
        }
    }

    struct Temporary(PathBuf);

    impl Temporary {
        fn new() -> Self {
            static SEQUENCE: AtomicU64 = AtomicU64::new(0);
            Self(std::env::temp_dir().join(format!(
                "hyprshell-bindings-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            )))
        }
    }

    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn spec(key: &str) -> Spec {
        Spec {
            modmask: 4,
            key: key.into(),
            exec: "hyprshell socat test".into(),
            release: false,
            desc: format!("test {key}"),
        }
    }

    #[test]
    fn legacy_restart_and_gui_reloads_are_idempotent() {
        let directory = Temporary::new();
        let mut transport = Mock::default();
        let desired = [spec("Tab")];
        reconcile(&mut transport, &desired, &directory.0, "instance").expect("first start");
        let original = transport.live.clone();
        transport.mutations.clear();
        reconcile(&mut transport, &desired, &directory.0, "instance").expect("second start");
        assert!(transport.mutations.is_empty());
        assert_eq!(transport.live, original);
    }

    #[test]
    fn removed_keys_are_retired_without_touching_callbacks() {
        let directory = Temporary::new();
        let mut transport = Mock::default();
        let callback = json!({"modmask":64,"key":"c","keycode":0,"dispatcher":"__lua","arg":"113","release":false,"submap":"","description":"desktop close"});
        transport.live.push(callback.clone());
        reconcile(
            &mut transport,
            &[spec("Tab"), spec("grave")],
            &directory.0,
            "instance",
        )
        .expect("first start");
        reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").expect("config update");
        assert_eq!(transport.live, [callback, fingerprint(&spec("Tab"))]);
        assert!(transport.mutations.contains(&"unbind:4:grave".into()));
    }

    #[test]
    fn shared_chords_in_other_submaps_abort_before_any_mutation() {
        let directory = Temporary::new();
        let mut transport = Mock::default();
        reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").expect("first start");
        let mut callback = fingerprint(&spec("Tab"));
        callback["submap"] = json!("desktop");
        callback["dispatcher"] = json!("__lua");
        callback["arg"] = json!("68");
        transport.live.push(callback);
        transport.mutations.clear();
        assert!(reconcile(&mut transport, &[], &directory.0, "instance").is_err());
        assert!(transport.mutations.is_empty());
        assert_eq!(transport.live.len(), 2);
    }

    #[test]
    fn missing_ledger_does_not_guess_ownership() {
        let directory = Temporary::new();
        let mut transport = Mock {
            live: vec![fingerprint(&spec("Tab"))],
            ..Default::default()
        };
        assert!(reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").is_err());
        assert!(transport.mutations.is_empty());
    }

    #[test]
    fn modified_fingerprints_are_not_treated_as_owned() {
        let directory = Temporary::new();
        let mut transport = Mock::default();
        reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").expect("first start");
        transport.live[0]["custom_future_flag"] = json!(false);
        transport.mutations.clear();
        assert!(reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").is_err());
        assert!(transport.mutations.is_empty());
    }

    #[test]
    fn failed_bind_stops_without_falling_back_or_claiming_ownership() {
        let directory = Temporary::new();
        let mut transport = Mock {
            fail_bind: true,
            ..Default::default()
        };
        assert!(reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").is_err());
        assert!(transport.mutations.is_empty());
        let store = Store::open(&directory.0, "instance").expect("store");
        assert!(store.load().expect("ledger").entries.is_empty());
    }

    #[test]
    fn lua_failure_never_uses_legacy_registration() {
        let directory = Temporary::new();
        let mut transport = Mock {
            lua_available: true,
            fail_lua: true,
            ..Default::default()
        };
        assert!(reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").is_err());
        assert_eq!(transport.mutations, ["lua"]);
        assert!(!directory.0.exists());
    }

    #[test]
    fn unknown_probe_failure_is_not_a_legacy_capability_signal() {
        let directory = Temporary::new();
        let mut transport = Mock {
            fail_probe: true,
            ..Default::default()
        };
        assert!(reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").is_err());
        assert!(transport.mutations.is_empty());
        assert!(!directory.0.exists());
    }

    #[test]
    fn externally_reloaded_bindings_are_registered_again() {
        let directory = Temporary::new();
        let mut transport = Mock::default();
        reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").expect("first start");
        transport.live.clear();
        reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance")
            .expect("after compositor reload");
        assert_eq!(transport.live.len(), 1);
    }

    #[test]
    fn recorded_duplicate_bindings_are_collapsed() {
        let directory = Temporary::new();
        let store = Store::open(&directory.0, "instance").expect("store");
        let entry = Owned {
            spec: spec("Tab"),
            fingerprint: fingerprint(&spec("Tab")),
        };
        store
            .save(&Ledger {
                version: 1,
                instance: "instance".into(),
                entries: vec![entry.clone(), entry],
            })
            .expect("ledger");
        let mut transport = Mock {
            live: vec![fingerprint(&spec("Tab")); 2],
            ..Default::default()
        };
        reconcile_legacy(&mut transport, &[spec("Tab")], &store).expect("deduplication");
        assert_eq!(transport.live.len(), 1);
    }

    #[test]
    fn additional_identical_bindings_are_not_claimed_by_one_ownership_record() {
        let directory = Temporary::new();
        let mut transport = Mock::default();
        reconcile(&mut transport, &[spec("Tab")], &directory.0, "instance").expect("first start");
        transport.live.push(fingerprint(&spec("Tab")));
        transport.mutations.clear();
        assert!(reconcile(&mut transport, &[], &directory.0, "instance").is_err());
        assert!(transport.mutations.is_empty());
        assert_eq!(transport.live.len(), 2);
    }

    #[test]
    fn invalid_ledgers_and_instance_names_fail_closed() {
        let directory = Temporary::new();
        assert!(Store::open(&directory.0, "../desktop").is_err());
        let store = Store::open(&directory.0, "instance").expect("store");
        fs::write(&store.path, b"invalid json").expect("corrupt ledger");
        let mut transport = Mock::default();
        assert!(reconcile_legacy(&mut transport, &[spec("Tab")], &store).is_err());
        assert!(transport.mutations.is_empty());
    }

    #[test]
    fn all_lua_string_bytes_are_escaped_without_json_unicode_syntax() {
        assert_eq!(lua_string("\"\n\\é"), "\"\\034\\010\\092\\195\\169\"");
    }
}
