//! Tracker datasets read from the working tree: modules, the node catalog and the design screens.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// One row per module: its layer, path, size and test count.
pub(super) fn modules(root: &Path) -> Value {
    let mut rows = Vec::new();
    let mut add = |layer: &str, name: String, path: String, about: String| {
        let (files, lines, tests) = measure(&root.join(&path));
        rows.push(json!({"layer": layer, "module": name, "path": path, "about": about,
            "files": files, "lines": lines, "tests": tests}));
    };
    let mut crates: Vec<String> = fs::read_dir(root.join("crates"))
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    crates.sort();
    for c in crates {
        let layer = match c.as_str() {
            "il-store" => "Database",
            "il-core" | "il-sdk" => "Contracts",
            "il-nodes" | "il-connectors" | "il-media" => "Nodes & apps",
            _ => "Backend",
        };
        let about = fs::read_to_string(root.join(format!("crates/{c}/src/lib.rs")))
            .ok()
            .and_then(|s| s.lines().next().and_then(|l| l.strip_prefix("//!")).map(|l| l.trim().to_string()))
            .unwrap_or_default();
        add(layer, c.clone(), format!("crates/{c}"), about);
    }
    add("Database", "migrations".into(), "migrations".into(), "SQL migrations, one file per change".into());
    for (dir, about) in [
        ("api", "API client, SSE helper, fixture mode and generated types"),
        ("design", "Tokens, fonts, base components and the app shell"),
        ("canvas", "Node canvas (Svelte Flow), node cards, edges, build state"),
        ("forms", "Settings forms generated from node JSON Schemas"),
        ("panel", "Node panel: settings, input and output"),
        ("previews", "Output preview renderers, one file each"),
        ("stores", "Shared client state"),
    ] {
        add("Frontend", format!("web/{dir}"), format!("web/src/lib/{dir}"), about.into());
    }
    add("Frontend", "web/routes".into(), "web/src/routes".into(), "Screens, one folder per route".into());
    add("Tests & tooling", "web/e2e".into(), "web/e2e".into(), "Playwright tests in fixture mode".into());
    add("Tests & tooling", "web/fixtures".into(), "web/fixtures".into(), "Recorded API answers and SSE streams".into());
    add(
        "Tests & tooling",
        "xtask".into(),
        "xtask".into(),
        "Repo automation: check, gen-types, catalog, progress".into(),
    );
    Value::Array(rows)
}

/// Source files, lines and tests under `dir` (generated types excluded).
fn measure(dir: &Path) -> (u64, u64, u64) {
    let (mut files, mut lines, mut tests) = (0, 0, 0);
    let Ok(entries) = fs::read_dir(dir) else { return (0, 0, 0) };
    for e in entries.filter_map(|e| e.ok()) {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !matches!(name.as_str(), "types" | "node_modules" | "target") {
                let (f, l, t) = measure(&p);
                (files, lines, tests) = (files + f, lines + l, tests + t);
            }
            continue;
        }
        let ext = p.extension().and_then(|x| x.to_str()).unwrap_or_default();
        if !matches!(ext, "rs" | "ts" | "svelte" | "css" | "sql" | "json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&p) else { continue };
        files += 1;
        lines += text.lines().count() as u64;
        tests += text
            .lines()
            .map(str::trim_start)
            .filter(|l| {
                l.starts_with("#[test]")
                    || l.starts_with("#[tokio::test")
                    || ((l.starts_with("it(") || l.starts_with("test(")) && ext == "ts")
            })
            .count() as u64;
    }
    (files, lines, tests)
}

/// The node catalog: one row per node type with its implementation status.
pub(super) fn nodes(root: &Path) -> anyhow::Result<Value> {
    let catalog: Value = serde_json::from_str(&fs::read_to_string(root.join("web/fixtures/api/nodes.json"))?)?;
    let rows = catalog["nodes"].as_array().cloned().unwrap_or_default();
    Ok(Value::Array(
        rows.iter()
            .map(|n| {
                json!({"type": n["typeId"], "title": n["title"], "category": n["category"],
                "app": n["app"], "status": n["status"]})
            })
            .collect(),
    ))
}

/// The design screens from `MAPPING.md`, with whether each route is built, a placeholder or missing.
pub(super) fn screens(root: &Path, mapping: &str, tasks: &[Value]) -> Value {
    let rows = super::table(mapping, "## Screens");
    let rows = rows.as_array().cloned().unwrap_or_default();
    let mut out: Vec<Value> = Vec::new();
    for r in rows {
        let id = r["id"].as_str().unwrap_or_default().to_string();
        if !id.starts_with(['S', 'M']) || !id[1..].chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let path = r["webSrcPath"].as_str().unwrap_or_default().to_string();
        let status = match path.strip_prefix("same as ").and_then(|s| s.split_whitespace().next()) {
            Some(other) => out.iter().find(|s| s["id"] == other).map_or(json!("missing"), |s| s["status"].clone()),
            None => json!(route_status(root, &path)),
        };
        let task = r["task"].as_str().unwrap_or_default().split_whitespace().next().unwrap_or_default().to_string();
        let task_status = tasks.iter().find(|t| t["id"] == task.as_str()).map_or(Value::Null, |t| t["status"].clone());
        out.push(json!({"id": id, "screen": r["designScreenDesignRoute"], "route": r["prdRoute"], "path": path,
            "task": task, "taskStatus": task_status, "status": status, "image": screen_image(&id)}));
    }
    Value::Array(out)
}

/// `built`, `placeholder` or `missing` for a route path under `web/src`.
fn route_status(root: &Path, path: &str) -> &'static str {
    let mut p = root.join("web/src").join(path.trim_end_matches('/'));
    if p.is_dir() {
        p = p.join("+page.svelte");
    }
    match fs::read_to_string(&p) {
        Ok(s) if s.contains("Placeholder") => "placeholder",
        Ok(_) => "built",
        Err(_) => "missing",
    }
}

/// The design PNG for a screen ID, under `design/interlock/screens/`.
fn screen_image(id: &str) -> Value {
    let file = match id {
        "S1" => "Landing",
        "S2" => "NewBackend",
        "S3" => "Building",
        "S4" => "Connect",
        "S5" => "Playground",
        "S6" => "Overview",
        "S7" => "Backends",
        "S8" => "Demo",
        "M1" => "MLanding",
        "M2" => "MBuilding",
        "M3" => "MConnect",
        "M4" => "MPlayground",
        _ => return Value::Null,
    };
    json!(format!("design/interlock/screens/{file}.png"))
}
