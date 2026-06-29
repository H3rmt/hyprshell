use anyhow::Context;
use std::fmt::Write;
use tracing::{debug, info, warn};

const MODE: spdx::ParseMode = spdx::ParseMode {
    allow_deprecated: true,
    allow_slash_as_or_operator: false,
    allow_imprecise_license_names: false,
    allow_postfix_plus_on_gpl: false,
    allow_unknown: false,
};

pub fn gen_licenses(licenses: &[String]) -> anyhow::Result<String> {
    let licenses = licenses
        .iter()
        .filter_map(|license| match spdx::Licensee::parse_mode(license, MODE) {
            Ok(licenseee) => Some(licenseee),
            Err(error) => {
                warn!("invalid license passed: {error:?}");
                None
            }
        })
        .collect::<Vec<_>>();
    info!("allowing {} licenses", licenses.len());
    let cfg = cargo_about::licenses::config::Config {
        accepted: licenses,
        ..Default::default()
    };
    debug!("gathering crates info");
    let crates = cargo_about::get_all_crates(
        krates::Utf8Path::new("Cargo.toml"),
        false,
        true,
        vec![],
        false,
        krates::LockOptions {
            locked: true,
            frozen: false,
            offline: false,
        },
        &cfg,
        &[],
    )
    .context("Failed to get all crates for licenses")?;
    debug!("gathering licenses info");
    let store =
        cargo_about::licenses::store_from_cache().context("failed to load license store")?;
    debug!("gathering summary info");
    let summary = cargo_about::licenses::Gatherer::with_store(std::sync::Arc::new(store))
        .with_confidence_threshold(0.8)
        .with_max_depth(Some(1))
        .gather(&crates, &cfg, None);
    let kcs = std::collections::BTreeMap::new();
    let mut files = codespan::Files::new();
    debug!("resolving licenses");
    let resolved =
        cargo_about::licenses::resolution::resolve(&summary, &cfg.accepted, &kcs, &mut files, true);
    debug!("generating licenses");
    let out = cargo_about::generate::generate(&summary, &resolved, move |diags| {
        for diag in diags {
            warn!("{diag:?}");
        }
    })?;

    let mut output = String::new();

    // Generate overview section
    output.push_str("## Overview of licenses:\n\n");
    for info in out.overview {
        let _ = output.write_fmt(format_args!("* {} ({})\n", info.name, info.count));
    }
    // Generate detailed license section
    output.push_str("\n## All license texts:\n");
    for info in out.licenses {
        let _ = output.write_fmt(format_args!(
            "\n### {} ({})\n\n#### Used by:\n\n",
            info.name, info.id
        ));
        for cr in info.used_by {
            let _ = output.write_fmt(format_args!(
                "* [{} {}]({})\n",
                cr.krate.name,
                cr.krate.version,
                match cr.krate.repository {
                    Some(ref repo) => repo.clone(),
                    None => format!("https://crates.io/crates/{}", cr.krate.name),
                }
            ));
        }
        let _ = output.write_fmt(format_args!(
            "\n```\n{}\n```\n--------------------------------------------------------------------------------\n",
            info.text
        ));
    }
    Ok(output)
}
