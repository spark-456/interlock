//! `cargo xtask <command>` — repo automation that works the same on Windows and Linux.
//!
//! - `dev`        server (8080) + web dev server with API proxy
//! - `check`      fmt, clippy -D warnings, tests, check-deps, type/catalog drift, web check + tests
//! - `check-deps` enforce the crate dependency direction (PRD §7.2)
//! - `gen-types`  ts-rs → web/src/lib/api/types, JSON Schemas → contracts/
//! - `catalog`    docs/NODES.md + web/fixtures/api/nodes.json from registered manifests
//! - `progress`   the progress tracker's data files, into target/progress/

mod deps;
mod generate;
mod progress;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{Context, bail};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = root();
    std::env::set_current_dir(&root)?;
    match args.first().map(String::as_str) {
        Some("dev") => dev(),
        Some("check") => check(args.iter().any(|a| a == "--no-web")),
        Some("check-deps") => deps::check(),
        Some("gen-types") => generate::types(&root.join("web/src/lib/api/types"), &root.join("contracts")),
        Some("catalog") => generate::catalog(&root.join("docs/NODES.md"), &root.join("web/fixtures/api/nodes.json")),
        Some("progress") => progress::write(&root, &root.join("target/progress")),
        _ => {
            eprintln!("usage: cargo xtask <dev | check [--no-web] | check-deps | gen-types | catalog | progress>");
            Ok(())
        }
    }
}

/// Repo root (xtask lives one level below it). Prefers the runtime `CARGO_MANIFEST_DIR` that `cargo run`
/// sets, so a binary built from another worktree (shared target dir) still points at this checkout.
pub fn root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from);
    let manifest_dir = manifest_dir.unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    manifest_dir.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."))
}

/// A command for a tool; on Windows `.cmd` shims (pnpm) need `cmd /C`.
pub fn tool(name: &str) -> Command {
    if cfg!(windows) && name == "pnpm" {
        let mut c = Command::new("cmd");
        c.args(["/C", "pnpm"]);
        c
    } else {
        Command::new(name)
    }
}

/// Runs a command, failing with its label if it fails.
pub fn run(label: &str, cmd: &mut Command) -> anyhow::Result<()> {
    eprintln!("\n==> {label}");
    let status: ExitStatus = cmd.status().with_context(|| format!("starting `{label}`"))?;
    if !status.success() {
        bail!("`{label}` failed ({status})");
    }
    Ok(())
}

fn check(skip_web: bool) -> anyhow::Result<()> {
    run("cargo fmt --check", Command::new("cargo").args(["fmt", "--all", "--check"]))?;
    run(
        "cargo clippy",
        Command::new("cargo").args(["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
    )?;
    run("cargo test", Command::new("cargo").args(["test", "--workspace", "--quiet"]).env("IL_PROVIDERS", "fake"))?;
    eprintln!("\n==> check-deps");
    deps::check()?;
    eprintln!("\n==> type + catalog drift");
    generate::drift()?;
    if skip_web {
        return Ok(());
    }
    if !Path::new("web/node_modules").exists() {
        run("pnpm install", tool("pnpm").args(["-C", "web", "install", "--frozen-lockfile"]))?;
    }
    run("pnpm check", tool("pnpm").args(["-C", "web", "check"]))?;
    run("pnpm test", tool("pnpm").args(["-C", "web", "test"]))?;
    eprintln!("\nAll checks passed.");
    Ok(())
}

fn dev() -> anyhow::Result<()> {
    eprintln!("Starting il-server on :8080 and the web dev server (proxying /api, /v1, /mcp, /hooks, /w)…");
    let mut server = Command::new("cargo").args(["run", "-p", "il-server"]).spawn().context("starting il-server")?;
    let mut web = tool("pnpm").args(["-C", "web", "dev"]).spawn().context("starting web dev server")?;
    let web_status = web.wait()?;
    let _ = server.kill();
    if !web_status.success() {
        bail!("web dev server exited with {web_status}");
    }
    Ok(())
}
