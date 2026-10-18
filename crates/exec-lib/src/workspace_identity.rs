//! Translate compositor identities to process-local UI keys without inventing
//! numeric compositor IDs or conflating named and special workspaces.
use anyhow::{Context, bail};
use core_lib::WorkspaceId;
use hyprland::data::{Workspace, WorkspaceBasic};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkspaceTarget {
    Number(i64),
    Named(String),
    Special(String),
}

fn target(id: Option<i64>, address: &str, kind: &str) -> anyhow::Result<WorkspaceTarget> {
    if let Some(id) = id {
        if id < i64::from(i32::MIN) || id > i64::from(u32::MAX) {
            bail!("workspace numeric ID outside legacy i32/current u32 range");
        }
        return Ok(WorkspaceTarget::Number(id));
    }
    match (kind, address) {
        ("numbered", address) => {
            let number: u32 = address
                .parse()
                .context("invalid numbered workspace address")?;
            Ok(WorkspaceTarget::Number(i64::from(number)))
        }
        ("normal" | "named", address) if !address.is_empty() => {
            Ok(WorkspaceTarget::Named(address.to_owned()))
        }
        ("special", address) if address.starts_with("special:") && address.len() > 8 => {
            Ok(WorkspaceTarget::Special(address[8..].to_owned()))
        }
        _ => bail!(
            "workspace has no complete compositor identity: type={kind:?}, address={address:?}"
        ),
    }
}

// Never recycle a key for a different address: open UI selections can outlive
// one collection snapshot. The daemon's lifetime owns this small registry.
#[derive(Default)]
struct Identities {
    keys: HashMap<WorkspaceTarget, WorkspaceId>,
    targets: HashMap<WorkspaceId, WorkspaceTarget>,
    named_count: i64,
    special_count: i64,
}

impl Identities {
    fn intern(&mut self, target: WorkspaceTarget) -> anyhow::Result<WorkspaceId> {
        if let WorkspaceTarget::Number(number) = target {
            return Ok(number);
        }
        if let Some(key) = self.keys.get(&target) {
            return Ok(*key);
        }
        let key = match &target {
            WorkspaceTarget::Named(_) => {
                self.named_count = self
                    .named_count
                    .checked_add(1)
                    .context("named workspace key exhaustion")?;
                i64::from(u32::MAX)
                    .checked_add(self.named_count)
                    .context("named workspace key exhaustion")?
            }
            WorkspaceTarget::Special(_) => {
                self.special_count = self
                    .special_count
                    .checked_add(1)
                    .context("special workspace key exhaustion")?;
                i64::from(i32::MIN)
                    .checked_sub(self.special_count)
                    .context("special workspace key exhaustion")?
            }
            WorkspaceTarget::Number(_) => unreachable!(),
        };
        self.keys.insert(target.clone(), key);
        self.targets.insert(key, target);
        Ok(key)
    }

    fn resolve(&self, key: WorkspaceId) -> anyhow::Result<WorkspaceTarget> {
        if (i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(&key) {
            return Ok(WorkspaceTarget::Number(key));
        }
        self.targets
            .get(&key)
            .cloned()
            .context("unknown process-local workspace key")
    }
}

fn identities() -> &'static Mutex<Identities> {
    static IDENTITIES: OnceLock<Mutex<Identities>> = OnceLock::new();
    IDENTITIES.get_or_init(|| Mutex::new(Identities::default()))
}

pub fn full_key(workspace: &Workspace) -> anyhow::Result<WorkspaceId> {
    key(workspace.id, &workspace.address, &workspace.workspace_type)
}

pub fn basic_key(workspace: &WorkspaceBasic) -> anyhow::Result<WorkspaceId> {
    key(workspace.id, &workspace.address, &workspace.workspace_type)
}

fn key(id: Option<i64>, address: &str, kind: &str) -> anyhow::Result<WorkspaceId> {
    let target = target(id, address, kind)?;
    identities()
        .lock()
        .map_err(|_| anyhow::anyhow!("workspace identity mutex poisoned"))?
        .intern(target)
}

pub fn resolve(key: WorkspaceId) -> anyhow::Result<WorkspaceTarget> {
    identities()
        .lock()
        .map_err(|_| anyhow::anyhow!("workspace identity mutex poisoned"))?
        .resolve(key)
}

#[cfg(test)]
mod workspace_identity_tests {
    use super::*;

    #[test]
    fn transitional_schema_and_display_renames_preserve_identity() {
        let mut ids = Identities::default();
        let named = target(None, "work", "named").expect("valid test fixture");
        assert_eq!(
            named,
            target(None, "work", "normal").expect("valid test fixture")
        );
        assert_eq!(
            target(None, "2", "numbered").expect("valid test fixture"),
            WorkspaceTarget::Number(2)
        );
        let key = ids.intern(named).expect("valid test fixture");
        let original = WorkspaceBasic {
            id: None,
            address: "work".to_owned(),
            workspace_type: "normal".to_owned(),
            name: "original label".to_owned(),
        };
        let renamed = WorkspaceBasic {
            name: "a different label".to_owned(),
            ..original.clone()
        };
        assert_eq!(
            basic_key(&original).expect("valid test fixture"),
            basic_key(&renamed).expect("valid test fixture")
        );
        let named_special = ids
            .intern(target(None, "special:work", "named").expect("valid test fixture"))
            .expect("valid test fixture");
        let special = ids
            .intern(target(None, "special:work", "special").expect("valid test fixture"))
            .expect("valid test fixture");
        assert_ne!(key, named_special);
        assert_ne!(named_special, special);
    }

    #[test]
    fn native_numeric_ids_and_signs_are_preserved_without_collisions() {
        let mut ids = Identities::default();
        for value in [
            i64::from(i32::MIN),
            -99,
            -1,
            0,
            1,
            i64::from(i32::MAX),
            i64::from(u32::MAX),
        ] {
            let key = ids
                .intern(target(Some(value), "", "").expect("valid test fixture"))
                .expect("valid test fixture");
            assert_eq!(key, value);
            assert_eq!(
                ids.resolve(key).expect("valid test fixture"),
                WorkspaceTarget::Number(value)
            );
        }
        let named = ids
            .intern(target(None, "work", "normal").expect("valid test fixture"))
            .expect("valid test fixture");
        let special = ids
            .intern(target(None, "special:work", "special").expect("valid test fixture"))
            .expect("valid test fixture");
        assert!(named > i64::from(u32::MAX));
        assert!(special < i64::from(i32::MIN));
        assert_ne!(named, special);
        let default_special = ids
            .intern(target(None, "special:special", "special").expect("valid test fixture"))
            .expect("valid test fixture");
        assert_eq!(
            ids.resolve(default_special).expect("valid test fixture"),
            WorkspaceTarget::Special("special".to_owned())
        );
    }

    #[test]
    fn canonical_identity_survives_reordering_and_other_workspace_creation() {
        let mut ids = Identities::default();
        let work = target(None, "work", "normal").expect("valid test fixture");
        let scratch = target(None, "special:scratchpad", "special").expect("valid test fixture");
        let first = ids.intern(work.clone()).expect("valid test fixture");
        let second = ids.intern(scratch.clone()).expect("valid test fixture");
        ids.intern(target(None, "other", "normal").expect("valid test fixture"))
            .expect("valid test fixture");
        assert_eq!(
            ids.intern(scratch.clone()).expect("valid test fixture"),
            second
        );
        assert_eq!(ids.intern(work.clone()).expect("valid test fixture"), first);
        assert_eq!(ids.resolve(first).expect("valid test fixture"), work);
        assert_eq!(ids.resolve(second).expect("valid test fixture"), scratch);
    }

    #[test]
    fn incomplete_or_inconsistent_address_only_identifiers_fail_closed() {
        for (address, kind) in [
            ("", ""),
            ("", "normal"),
            ("work", "unknown"),
            ("work", "special"),
            ("special:", "special"),
        ] {
            assert!(target(None, address, kind).is_err());
        }
        assert!(
            Identities::default()
                .resolve(i64::from(u32::MAX) + 1)
                .is_err()
        );
    }
    #[test]
    fn numeric_looking_named_collection_identities_remain_distinct() {
        let mut ids = Identities::default();
        let numeric = ids
            .intern(target(Some(5), "5", "normal").expect("valid test fixture"))
            .expect("valid test fixture");
        let named = ids
            .intern(target(None, "5", "normal").expect("valid test fixture"))
            .expect("valid test fixture");
        let special = ids
            .intern(target(None, "special:5", "special").expect("valid test fixture"))
            .expect("valid test fixture");
        assert_eq!(numeric, 5);
        assert_ne!(numeric, named);
        assert_ne!(named, special);
        let WorkspaceTarget::Named(address) = ids.resolve(named).expect("valid test fixture")
        else {
            panic!("named identity lost");
        };
        assert_eq!(address, "5");
        assert_eq!(
            ids.resolve(special).expect("valid test fixture"),
            WorkspaceTarget::Special("5".to_owned())
        );
    }
}
