//! The bundle's generated text files: `.env.example`, `interlock.toml`, `compose.yaml`, the README
//! and `INTERLOCK.md`, each tailored to the flow.

use il_core::manifest::Category;
use il_core::{Flow, FlowKind};
use serde_json::Value;

use super::manifest::{Connection, EnvVar, Manifest, PROMPT_REF};
use crate::openai::slug;

/// The engine's address in the generated docs.
const LOCAL: &str = "http://localhost:8080";

/// Model providers and the env var holding each one's key (as `il-ai` reads them).
const PROVIDER_KEYS: [(&str, &str, &str); 3] = [
    ("tokenfactory", "NEBIUS_API_KEY", "Nebius Token Factory"),
    ("openrouter", "OPENROUTER_API_KEY", "OpenRouter"),
    ("groq", "GROQ_API_KEY", "Groq"),
];

fn var(name: &str, required: bool, about: impl Into<String>) -> EnvVar {
    EnvVar { name: name.into(), required, about: about.into() }
}

fn has(flow: &Flow, pred: impl Fn(&str) -> bool) -> bool {
    flow.nodes.iter().any(|n| !n.disabled && pred(&n.node_type))
}

/// Exactly the env vars this flow reads.
pub(super) fn env_vars(config: &Value, flow: &Flow, connections: &[Connection], webhooks: bool) -> Vec<EnvVar> {
    let mut vars = vec![var(
        "INTERLOCK_API_KEY",
        true,
        "Any long random value (32+ characters). Clients send it as `Authorization: Bearer …`.",
    )];
    let uses_models = has(flow, |t| {
        il_sdk::registry::node(t).is_some_and(|e| matches!(e.manifest().category, Category::Ai | Category::Knowledge))
    });
    let transcribes = has(flow, |t| t == "ai.transcribe");
    let primary = config.pointer("/models/provider").and_then(Value::as_str).unwrap_or("tokenfactory");
    let mut providers = vec![(primary, config.pointer("/models/roles/transcribe").is_some(), true)];
    for fallback in config.pointer("/models/fallbacks").and_then(Value::as_array).into_iter().flatten() {
        if let Some(name) = fallback["provider"].as_str() {
            providers.push((name, fallback.pointer("/roles/transcribe").is_some(), false));
        }
    }
    for (name, transcribe, main) in providers {
        let Some((_, env, label)) = PROVIDER_KEYS.iter().find(|(p, _, _)| *p == name) else { continue };
        if vars.iter().any(|v| v.name == *env) {
            continue;
        }
        let about = match (main, transcribe && transcribes) {
            (_, true) => format!("{label} key: models, and speech to text for Transcribe audio."),
            (true, false) => format!("{label} key for the models in interlock.toml."),
            (false, false) => format!("{label} key, a fallback when the main model provider is busy or down."),
        };
        vars.push(var(env, uses_models && (main || transcribe && transcribes), about));
    }
    let crawls = flow.nodes.iter().any(|n| {
        n.node_type == "knowledge.base"
            && n.settings["sources"].as_array().is_some_and(|s| s.iter().any(|s| s["kind"] == "url"))
    });
    if crawls {
        vars.push(var("TAVILY_API_KEY", true, "Reads the knowledge base's websites (tavily.com)."));
    }
    let runway = config
        .pointer("/providers/media")
        .and_then(Value::as_object)
        .is_some_and(|m| m.values().any(|v| v == "runway"));
    if runway && has(flow, |t| t.starts_with("ai.generate_")) {
        vars.push(var("RUNWAYML_API_SECRET", true, "Generates video and images with Runway."));
    }
    if !connections.is_empty() {
        vars.push(var(
            "IL_MASTER_KEY",
            true,
            "32 random bytes, base64 (`openssl rand -base64 32`). Encrypts the connections below at rest.",
        ));
    }
    for c in connections {
        for field in &c.fields {
            let what = if c.oauth { "OAuth refresh token" } else { &field.key };
            vars.push(var(&field.env, true, format!("{}: {what}, used by {}.", c.label, c.used_by.join(", "))));
        }
    }
    if connections.iter().any(|c| c.oauth && c.app == "google") {
        vars.push(var("GOOGLE_CLIENT_ID", true, "Your Google OAuth app, to refresh the Google sign-in."));
        vars.push(var("GOOGLE_CLIENT_SECRET", true, "Your Google OAuth app's secret."));
    }
    if webhooks {
        vars.push(var(
            "INTERLOCK_WEBHOOK_TOKEN",
            true,
            "Any long random value (24+ characters); webhooks arrive at /hooks/<flow id>/<this token>.",
        ));
    }
    vars
}

/// `.env.example`: every variable with a placeholder, never a value.
pub(super) fn env_example(flow: &Flow, manifest: &Manifest) -> String {
    let mut out = format!(
        "# {}: settings for the Interlock engine. Copy to .env and fill in; never commit .env.\n\
         # IL_PROVIDERS=fake runs everything offline with built-in fakes (no keys needed).\n\
         IL_PROVIDERS=real\n",
        flow.name
    );
    for v in &manifest.env {
        let optional = if v.required { "" } else { "(optional) " };
        let line = if v.required { format!("{}=", v.name) } else { format!("# {}=", v.name) };
        out.push_str(&format!("\n# {optional}{}\n{line}\n", v.about));
    }
    out
}

/// `interlock.toml`: the server's model, provider, engine and cap sections, without anything named
/// like a secret, and a headless server that keeps its data in `data/`.
pub(super) fn config_toml(config: &Value) -> Result<String, String> {
    let mut doc = toml::Table::new();
    for section in ["models", "providers", "tavily", "engine", "caps"] {
        if let Some(value) = config.get(section) {
            let value = toml::Value::try_from(scrub(value)).map_err(|e| e.to_string())?;
            doc.insert(section.into(), value);
        }
    }
    let body = toml::to_string(&doc).map_err(|e| e.to_string())?;
    Ok(format!(
        "# Interlock engine settings for this bundle. No secrets here: keys come from .env.\n\
         # Model roles and providers can be changed below; see the README.\n\n\
         [server]\n\
         host = \"127.0.0.1\"   # compose sets IL_HOST=0.0.0.0 inside the container\n\
         port = 8080\n\
         # Headless: every endpoint needs the API key or the webhook token. Set true to use the\n\
         # Interlock app at http://localhost:8080 on a trusted machine.\n\
         local_workspace = false\n\n\
         [storage]\n\
         path = \"data\"\n\n{body}"
    ))
}

/// The value without nulls (TOML has none) or keys named like secrets.
fn scrub(value: &Value) -> Value {
    let secret = |k: &str| {
        let k = k.to_ascii_lowercase();
        ["key", "secret", "token", "password"].iter().any(|s| k == *s || k.ends_with(&format!("_{s}")))
    };
    match value {
        Value::Object(map) => Value::Object(
            map.iter().filter(|(k, v)| !v.is_null() && !secret(k)).map(|(k, v)| (k.clone(), scrub(v))).collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().filter(|v| !v.is_null()).map(scrub).collect()),
        other => other.clone(),
    }
}

/// `compose.yaml`: builds the engine from source at the exporting version and loads this folder.
pub(super) fn compose(flow: &Flow, manifest: &Manifest) -> String {
    format!(
        "# Runs {name} with the open-source Interlock engine. The first `docker compose up` builds the\n\
         # engine from source (about 10 minutes); later starts take a second.\n\
         services:\n  \
           interlock:\n    \
             build: ${{INTERLOCK_SOURCE:-{source}}}#${{INTERLOCK_REF:-{git_ref}}}\n    \
             image: interlock-engine:local\n    \
             env_file: .env\n    \
             environment:\n      \
               IL_BUNDLE: /bundle\n      \
               IL_CONFIG: /bundle/interlock.toml\n      \
               IL_HOST: 0.0.0.0\n    \
             volumes:\n      \
               - ./:/bundle:ro\n      \
               - interlock-data:/data\n    \
             ports:\n      \
               - \"127.0.0.1:${{INTERLOCK_PORT:-8080}}:8080\"\n    \
             restart: unless-stopped\n\
         volumes:\n  \
           interlock-data:\n",
        name = flow.name,
        source = manifest.engine.source,
        git_ref = manifest.engine.git_ref,
    )
}

/// How the flow is reached once it runs locally.
fn usage(flow: &Flow, manifest: &Manifest) -> String {
    let mut out = String::new();
    if flow.kind == FlowKind::App {
        let s = slug(flow);
        out.push_str(&format!(
            "| Endpoint | Address |\n|---|---|\n\
             | OpenAI-compatible API (model `{s}`) | `{LOCAL}/v1/chat/completions` |\n\
             | MCP (streamable HTTP) | `{LOCAL}/mcp/{s}` |\n\
             | Chat widget | `<script src=\"{LOCAL}/w/{s}.js\"></script>` |\n\n\
             Every call sends `Authorization: Bearer $INTERLOCK_API_KEY` (the widget excepted).\n\n\
             ```sh\ncurl {LOCAL}/v1/chat/completions -H \"Authorization: Bearer $INTERLOCK_API_KEY\" \\\n  \
             -H \"Content-Type: application/json\" \\\n  \
             -d '{{\"model\":\"{s}\",\"messages\":[{{\"role\":\"user\",\"content\":\"Hello\"}}]}}'\n```\n\n\
             Code written against the hosted app works unchanged: point its base URL at `{LOCAL}/v1` and its \
             key at `INTERLOCK_API_KEY`.\n"
        ));
    }
    for hook in &manifest.webhooks {
        out.push_str(&format!(
            "\n**{}** receives webhooks at `{LOCAL}/hooks/{}/$INTERLOCK_WEBHOOK_TOKEN`.\n",
            hook.name, hook.flow_id
        ));
    }
    // Triggers other than requests (chat, API, MCP, webhooks, manual) start runs by themselves.
    let own = flow.nodes.iter().filter(|n| !n.disabled).any(|n| {
        il_sdk::registry::node(&n.node_type).is_some_and(|e| e.manifest().trigger)
            && !matches!(
                n.node_type.as_str(),
                "trigger.chat" | "trigger.webhook" | "trigger.manual" | "trigger.api_call" | "trigger.mcp_call"
            )
    });
    if own {
        out.push_str("\nIt starts on its own (schedules and app events) while the engine is running.\n");
    }
    out.push_str(&format!(
        "\nRuns, approvals and manual runs: the platform MCP at `{LOCAL}/mcp` with the same key \
         (`list_runs`, `get_run`, `list_approvals`, `decide_approval`, `run_flow`).\n"
    ));
    out
}

fn settings_table(manifest: &Manifest) -> String {
    let mut out = String::from("| Variable | Needed | What it is |\n|---|---|---|\n");
    for v in &manifest.env {
        out.push_str(&format!("| `{}` | {} | {} |\n", v.name, if v.required { "yes" } else { "optional" }, v.about));
    }
    out
}

fn steps_table(flow: &Flow) -> String {
    let mut out = String::from("| Step | Id | Type | Prompt |\n|---|---|---|---|\n");
    for n in &flow.nodes {
        let prompts: Vec<String> = n
            .settings
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(_, v)| v.as_str().filter(|v| v.starts_with(PROMPT_REF)).map(|v| format!("`{}`", &v[1..])))
            .collect();
        out.push_str(&format!("| {} | `{}` | `{}` | {} |\n", n.name, n.id, n.node_type, prompts.join(", ")));
    }
    out
}

const KNOWLEDGE_ROW: &str =
    "| `knowledge/` | Saved knowledge bases the steps search, as text (rebuilt on first start) |\n";

/// README.md for the person who runs it.
pub(super) fn readme(flow: &Flow, manifest: &Manifest) -> String {
    let goal = flow.meta.goal.as_deref().map(|g| format!("> {g}\n\n")).unwrap_or_default();
    let what = if flow.kind == FlowKind::App { "AI app" } else { "automation" };
    let flow_file = &manifest.flows[0];
    let e = &manifest.engine;
    format!(
        "# {name}\n\n{goal}This folder runs the {what} **{name}** (version {version}) on your own machines with the \
         open-source [Interlock]({repo}) engine: no calls to the hosted service, and no outside calls except the \
         model and API providers you set in `.env`.\n\n\
         ## Run it\n\n\
         1. `cp .env.example .env` and fill in the values (see [Settings](#settings)).\n\
         2. `docker compose up -d` (the first start builds the engine from source, about 10 minutes).\n\
         3. `curl {LOCAL}/healthz` answers `{{\"ok\":true,…}}`.\n\n\
         Without Docker, install the engine with Rust (`cargo install --locked --git {source} il-server`), export \
         the variables from `.env`, and run `IL_BUNDLE=. IL_CONFIG=interlock.toml il-server` in this folder.\n\n\
         To try it with nothing leaving your machine, set `IL_PROVIDERS=fake` in `.env`: built-in fakes answer \
         every model and app call, and no keys are needed.\n\n\
         ## Use it\n\n{usage}\n\
         ## Settings\n\n{settings}\n\
         ## What's inside\n\n\
         | Path | What it is |\n|---|---|\n\
         | `{flow_file}` | The flow: steps, settings and connections between them |\n\
         | `prompts/` | Each step's instructions or prompt, one Markdown file per prompt |\n\
         | `files/` | Documents, API specs and samples the steps use |\n\
         {knowledge}\
         | `interlock.toml` | Models, limits and spend caps (no secrets) |\n\
         | `interlock.bundle.json` | What the engine loads, and the env var behind each connection |\n\
         | `compose.yaml`, `.env.example` | How it runs, and the keys it needs |\n\
         | `INTERLOCK.md` | Notes for coding agents |\n\n\
         ## Steps\n\n{steps}\n\
         ## Change it\n\n\
         - Edit a prompt in `prompts/`, or steps and settings in `{flow_file}`, then `docker compose restart`. \
         Each change is saved as a new version.\n\
         - Or change it in Interlock, export it again and replace this folder (keep your `.env`).\n\
         - Models and spend caps are in `interlock.toml`. To build a newer engine, set `INTERLOCK_REF` in `.env` \
         to a tag or commit and run `docker compose build`.\n\n\
         ## Security\n\n\
         - Keys live only in `.env`, which git ignores. Connections are stored encrypted with `IL_MASTER_KEY`.\n\
         - The API and MCP need the API key and webhooks their token; the chat widget, as on the hosted app, \
         answers any page that embeds it. The port is published on 127.0.0.1 only; put a TLS proxy in front \
         before exposing it.\n\
         - To use the Interlock app at {LOCAL} on a trusted machine, set `local_workspace = true` in \
         `interlock.toml`.\n\n\
         Exported {date} · engine {version_engine} from `{source}` at `{git_ref}`.\n",
        name = flow.name,
        version = flow.version,
        repo = e.source.trim_end_matches(".git"),
        source = e.source,
        usage = usage(flow, manifest),
        settings = settings_table(manifest),
        steps = steps_table(flow),
        knowledge = if manifest.knowledge.is_empty() { "" } else { KNOWLEDGE_ROW },
        date = manifest.exported_at.format("%Y-%m-%d"),
        version_engine = e.version,
        git_ref = e.git_ref,
    )
}

/// INTERLOCK.md for coding agents working in a repository that contains the bundle.
pub(super) fn agent_guide(flow: &Flow, manifest: &Manifest) -> String {
    let calls = if flow.kind == FlowKind::App {
        let guide = crate::guide::generate(flow, LOCAL);
        guide.split_once('\n').map_or(guide.clone(), |(_, rest)| rest.trim_start().to_owned())
    } else {
        usage(flow, manifest)
    };
    let flow_file = &manifest.flows[0];
    format!(
        "# INTERLOCK.md: {name}\n\n\
         For coding agents. This folder is an Interlock flow exported to run self-hosted: the open-source engine \
         loads it on start (`IL_BUNDLE`) and serves the same endpoints as the hosted app. See README.md for setup.\n\n\
         ## Rules\n\n\
         - Never commit `.env` or write a key into any other file. A new key goes into `.env.example` as an \
         empty placeholder.\n\
         - Prompts live in `prompts/*.md`; the flow refers to them as `@prompts/<file>`. Keep those references.\n\
         - Keep step ids and `cred_…` ids: runs, connections and webhooks refer to them.\n\
         - Changes apply on restart (`docker compose restart`); the engine saves each as a new flow version.\n\n\
         ## Run and test\n\n\
         ```sh\ncp .env.example .env   # then fill it in\ndocker compose up -d\ncurl {LOCAL}/healthz\n```\n\n\
         For tests and CI, set `IL_PROVIDERS=fake`: models and apps are answered by offline fakes, and only \
         `INTERLOCK_API_KEY` (and `INTERLOCK_WEBHOOK_TOKEN`, if listed) need values.\n\n\
         ## Call it\n\n{calls}\n\
         ## Change it\n\n\
         `{flow_file}` is the flow: `nodes` (each step's `id`, `type`, `name`, `settings` and `credentials`) and \
         `edges` (`from`/`to` a node id and port). Settings are checked against each step type's schema when \
         the flow runs. Step types are described in the Interlock docs (`docs/NODES.md` in the engine source).\n\n\
         {steps}\n\
         ## Settings\n\n{settings}",
        name = flow.name,
        steps = steps_table(flow),
        settings = settings_table(manifest),
    )
}
