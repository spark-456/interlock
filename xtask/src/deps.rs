//! `check-deps`: enforces `il-core ← il-sdk ← implementations` (PRD §7.2) from `cargo metadata`.

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, bail};
use serde_json::Value;

/// Which workspace crates each crate may depend on (any dependency kind).
fn allowed(name: &str) -> Option<&'static [&'static str]> {
    const IMPL: &[&str] = &["il-core", "il-sdk"];
    Some(match name {
        "il-core" => &[],
        "il-sdk" => &["il-core"],
        "il-nodes" | "il-connectors" | "il-media" | "il-ai" | "il-store" => IMPL,
        "il-knowledge" => &["il-core", "il-sdk", "il-store"],
        "il-engine" | "il-architect" | "il-publish" => IMPL,
        // il-server wires everything; xtask is tooling.
        "il-server" | "xtask" => return None,
        _ => &[],
    })
}

/// External crates that must never appear in `il-core` (it has no IO and no async runtime).
const CORE_FORBIDDEN: &[&str] = &["tokio", "reqwest", "rusqlite", "axum", "async-trait"];

/// Runs the check.
pub fn check() -> anyhow::Result<()> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .context("running cargo metadata")?;
    if !out.status.success() {
        bail!("cargo metadata failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    let meta: Value = serde_json::from_slice(&out.stdout)?;
    let packages = meta["packages"].as_array().context("no packages")?;
    let mut deps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in packages {
        let name = p["name"].as_str().unwrap_or_default().to_string();
        let list =
            p["dependencies"].as_array().into_iter().flatten().filter_map(|d| d["name"].as_str()).map(String::from);
        deps.insert(name, list.collect());
    }
    let violations = violations(&deps);
    if !violations.is_empty() {
        for v in &violations {
            eprintln!("  dependency violation: {v}");
        }
        bail!("{} dependency-direction violation(s); see PRD §7.2", violations.len());
    }
    eprintln!("  dependency direction ok ({} crates)", deps.len());
    Ok(())
}

fn violations(deps: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut out = Vec::new();
    for (name, list) in deps {
        let Some(allowed) = allowed(name) else { continue };
        for dep in list {
            if dep.starts_with("il-") && !allowed.contains(&dep.as_str()) {
                out.push(format!("{name} → {dep}"));
            }
            if name == "il-core" && CORE_FORBIDDEN.contains(&dep.as_str()) {
                out.push(format!("il-core → {dep} (il-core must not do IO)"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect())).collect()
    }

    #[test]
    fn detects_engine_importing_store() {
        let v = violations(&map(&[("il-engine", &["il-core", "il-sdk", "il-store"])]));
        assert_eq!(v, ["il-engine → il-store"]);
    }

    #[test]
    fn allows_knowledge_store_and_server_anything() {
        let v =
            violations(&map(&[("il-knowledge", &["il-store", "il-sdk"]), ("il-server", &["il-store", "il-engine"])]));
        assert!(v.is_empty());
    }

    #[test]
    fn core_must_not_use_tokio() {
        assert_eq!(violations(&map(&[("il-core", &["tokio"])])).len(), 1);
    }
}
