# Interlock — Product Requirements Document

**Status:** v1.0 · hackathon build · 27 Oct 2026
**Deadline:** Nebius × NVIDIA Global AI Hackathon — Fri 30 Oct 2026, 10:00 AM PDT
**Companion files:** `AGENTS.md` (rules for every coding agent), `docs/WORKBOARD.md` (task packages and claims), `design/` (Claude Design exports)

---

## 1. Summary

Interlock is an end-to-end AI automation and backend builder. A user describes what they want — in text, by voice, or with documents and audio — and Interlock builds it as a working **flow** on a visual canvas: triggers, AI steps, knowledge, media, logic, app integrations, human approvals and outputs. Every flow is fully editable, swappable and testable, node by node, like Dify and n8n, but nobody starts from a blank canvas, writes code or configures models.

Two kinds of flow share one canvas and one engine:

| Kind | Starts from | Delivered as |
|---|---|---|
| **Automation** | Schedule, webhook, new email, new file, app event, manual | Activate toggle; runs in the background with run history |
| **AI app** | Chat message, API call, MCP call | OpenAI-compatible REST API, MCP server, chat widget, integration guide |

Promise: **Describe it. Interlock builds it. You can change anything.**

## 2. Goals and non-goals

### Goals (hackathon build)

1. Goal → working flow in about two minutes, shown live on the canvas, with a test run producing real outputs.
2. A Dify/n8n-class editor: node palette, canvas, node panel with auto-generated settings forms, real input/output previews, Replace with…, Ask Interlock diff edits.
3. Three hero flows working end to end: product promo videos, support inbox replies, support agent from documents and audio (section 4).
4. AI apps published as an OpenAI-compatible API, a per-flow MCP server, a platform MCP for coding agents, and a chat widget.
5. A modular codebase where new nodes, connectors, model providers and preview renderers are added without editing the engine, the architect or the UI shell.
6. Hosted online; everyone signs in and works in their own workspace (demo mode was removed on 30 Oct 2026).
7. Full compliance with the hackathon rules (section 13).

### Non-goals (this build)

Multi-region scaling, enterprise SSO, billing, a marketplace, real-time multiplayer editing, self-repairing flows, outcome simulation (see `docs/FUTURE.md`), and platform-approved posting to Instagram, TikTok, LinkedIn and X (those nodes exist but run in dry-run mode until the platform approves the app).

## 3. Users

- **Builders without code**: founders, marketers, support leads and operations people who want work automated.
- **Developers**: want an AI backend (agents + knowledge + tools) behind a stable API or MCP, integrated into their app by their coding agent.

## 4. Hero flows (acceptance scenarios)

Every module is done only when these flows work through it. Full sample data lives in `fixtures/` and in the design prompt.

### F-A · Weekly product promos (automation)

Request: "Every Monday, take our three newest Shopify products, make a short promo video for each with a voiceover, cut it for Reels, TikTok, Shorts and LinkedIn with captions, write the post text, and post once I approve in Slack."

Flow: Schedule → Shopify (sandbox store) get newest products → per item: AI write script → AI generate video → AI generate voice → Media merge → Media cut clips (9:16, 1:1, 16:9) → Media captions → AI write posts → Human approval (in-app + Slack) → post to YouTube (private), Bluesky, Discord (real); Instagram, TikTok, LinkedIn (dry-run) → Google Sheets log.

Acceptance: generated from the sentence; test run produces visible clips and post drafts (video and voice generation use the `recorded` provider; real adapters exist but are not live-tested); approval works on desktop and mobile; approved posts go out on the real connectors; a recorded run replays in demo mode.

### F-B · Support inbox replies (automation)

Request: "Reply to emails sent to support@ using our help center and policies. Send simple answers automatically; ask me in Slack before sending anything about refunds or complaints. Log everything in a sheet."

Flow: Gmail new email → AI classify (Question, Billing, Complaint, Spam) → Switch → Search knowledge → AI draft reply with sources → If confident and not refund/complaint → Gmail reply; else Slack approval → Gmail reply → Sheets log; Spam → Gmail label.

Acceptance: real Gmail (testing-mode OAuth) receives, classifies, drafts and replies; approvals route correctly; every run is visible with per-node data.

### F-C · Support agent from documents and audio (AI app)

Request: "A support chat agent that answers from these documents and can look up invoices through our API." Attachments: a help-center URL (crawled), three PDFs/DOCX, a 4-minute voice memo marked Instructions, an OpenAPI spec + key.

Flow: Chat trigger → AI decide (router) → Agent (Support) / Agent (Billing) with Knowledge base and OpenAPI tools → Guardrails → Respond.

Acceptance: knowledge base built (hybrid search); voice memo transcribed into instructions; tools generated from the spec (risky endpoints skipped); generated tests run and failures are fixed automatically; published API works with the official OpenAI SDK; MCP server works in Claude Code; widget works on a test page; platform MCP lets a coding agent fetch the integration guide.

## 5. Functional requirements

Priority: **P0** = must ship Friday; **P1** = ship if time allows. Each requirement names its owning module (section 7).

### 5.1 Create and architect (`il-architect`, web `create/`)

| ID | Requirement | P |
|---|---|---|
| AR-1 | Accept a goal (text or voice), attachments (files, links, audio) each marked Knowledge or Instructions, and optional app/API hints | P0 |
| AR-2 | Ask at most three clarifying questions, only when a required value is missing | P1 |
| AR-3 | Compile the goal into a valid Flow JSON using only node types from the live node catalog (`GET /api/nodes`), with settings that validate against each node's schema | P0 |
| AR-4 | Write all prompts, map data between nodes with expressions, group nodes, and lay out the canvas | P0 |
| AR-5 | Detect required connections and mark nodes that need them | P0 |
| AR-6 | Test-run with sample data, detect failures, attempt automatic fixes (max 2 rounds), report what was fixed | P0 |
| AR-7 | Stream every stage as build events so the canvas animates from real progress | P0 |
| AR-8 | Ask Interlock: turn a change request into a JSON-Patch diff against the current flow, shown for accept/discard | P0 |
| AR-9 | For AI apps: generate guardrails and a test suite (questions and tool tasks) from the knowledge and API | P0 |
| AR-10 | Validation and repair loop: invalid model output is repaired by re-prompting with the validation errors (max 3 attempts) | P0 |

### 5.2 Editor (web `canvas/`, `panel/`, `forms/`, `previews/`)

| ID | Requirement | P |
|---|---|---|
| ED-1 | Canvas with node cards, edges, groups, minimap, zoom, fit, tidy, undo/redo, list view | P0 |
| ED-2 | Node palette generated from the node catalog; search; drag to add; drop on an edge to insert | P0 |
| ED-3 | Node panel with tabs Settings / Input / Output; the Settings form is rendered from the node's JSON Schema (no per-node UI code required) | P0 |
| ED-4 | Field mapper: insert expressions referencing earlier nodes' outputs as chips | P0 |
| ED-5 | Output previews via a renderer registry: text, email, image, video, audio, table, JSON | P0 |
| ED-6 | Replace with… keeps position and compatible settings and mappings | P0 |
| ED-7 | Run a single step with pinned or last input | P1 |
| ED-8 | Versions: every save creates a version; restore any version | P1 |
| ED-9 | Ask Interlock bar and diff view (added/changed/removed) | P0 |

### 5.3 Engine and runs (`il-engine`)

| ID | Requirement | P |
|---|---|---|
| EN-1 | Execute a flow graph: per-item or batch nodes, branches with named outputs, merges, parallel branches with a concurrency limit | P0 |
| EN-2 | Retries with backoff per node; per-node timeout; cancellation | P0 |
| EN-3 | Pause at Human nodes, persist state, resume on decision (survives restart) | P0 |
| EN-4 | Emit ordered events for every state change (section 8.4) to an event store and to live subscribers | P0 |
| EN-5 | Triggers: manual, schedule (cron with timezone), webhook, polling app triggers, chat/API/MCP invocation | P0 |
| EN-6 | Spend caps: per run, per flow per day, per workspace per day; a run pauses with a clear state when a cap is hit | P0 |
| EN-7 | Run history with per-node inputs/outputs and binary file references | P0 |
| EN-8 | Replay a recorded run (events + files) with a visible replay label | P0 |

### 5.4 Knowledge (`il-knowledge`)

| ID | Requirement | P |
|---|---|---|
| KN-1 | Ingest PDF (text layer), DOCX, Markdown, TXT, CSV, pasted text | P0 |
| KN-2 | Crawl/extract websites through Tavily (cap 50 pages per source) | P0 |
| KN-3 | Transcribe audio through a transcription provider; mark result as Knowledge or Instructions | P0 |
| KN-4 | Chunk with document title, section and date metadata; embed via the model gateway | P0 |
| KN-5 | Hybrid search: vector (sqlite-vec) + keyword (FTS5), fused ranking, top-k with sources | P0 |
| KN-6 | Add documents to an existing knowledge base via UI, API and MCP | P1 |

### 5.5 Nodes and connectors (`il-nodes`, `il-connectors`, `il-media`)

P0 node set (the catalog the architect may use):

- **Triggers**: Manual, Schedule, Webhook, Chat message, API call, MCP call, New email (Gmail), App event (Shopify sandbox: new product).
- **AI**: Write, Summarise, Classify, Extract fields, Translate, Decide, Agent (tool loop with knowledge and tools), Describe image, Transcribe audio, Generate image, Generate video, Generate voice.
- **Knowledge**: Knowledge base, Search knowledge, Add to knowledge.
- **Media**: Cut clips, Resize for platform, Add captions, Merge audio and video, Thumbnail.
- **Logic**: If, Switch, Loop over items (split a list field into items so following nodes run per item), Merge, Wait, Stop.
- **Data**: Set fields, Template, HTTP request, Use any API (OpenAPI), Parse document.
- **Human**: Approval (in-app, Slack), Ask a question.
- **Output**: Respond, Send notification.
- **Apps (real)**: Gmail (new email, reply, send, label), Google Sheets (append row), Slack (send message, approval), Discord (send), Telegram (send), Bluesky (post), YouTube (upload, private), Shopify sandbox (get products).
- **Apps (dry-run until platform approval)**: Instagram, TikTok, LinkedIn, X. They validate inputs, render a preview of the post, and record "dry-run" in the output.

P1: Outlook, Notion, Google Drive new file, HubSpot, Airtable, Code node (sandboxed), Generate music, Watermark.

**Where each node lives** (one module per node; this is also the ownership map in the workboard):

| Nodes | Crate folder |
|---|---|
| Manual, Schedule, Webhook, Chat message, API call, MCP call | `il-nodes/src/trigger/` |
| Write, Summarise, Classify, Extract fields, Translate, Decide, Agent, Describe image, Transcribe audio | `il-nodes/src/ai/` |
| Knowledge base, Search knowledge, Add to knowledge | `il-nodes/src/knowledge/` |
| If, Switch, Loop over items, Merge, Wait, Stop | `il-nodes/src/logic/` |
| Set fields, Template, HTTP request, Parse document | `il-nodes/src/data/` |
| Approval, Ask a question | `il-nodes/src/human/` |
| Respond, Send notification | `il-nodes/src/output/` |
| Generate image, Generate video, Generate voice | `il-media/src/gen/` |
| Cut clips, Resize for platform, Add captions, Merge audio and video, Thumbnail | `il-media/src/ops/` |
| Use any API (OpenAPI) | `il-connectors/src/openapi/` |
| App nodes (Gmail, Sheets, YouTube, Slack, Discord, Telegram, Bluesky, Shopify, Instagram/TikTok/LinkedIn/X dry-run) | `il-connectors/src/<app>/` (`social_dry_run/` for the four dry-run platforms) |

### 5.6 Publish and connect (`il-publish`)

| ID | Requirement | P |
|---|---|---|
| PB-1 | OpenAI-compatible `POST /v1/chat/completions` (streaming and non-streaming) and `GET /v1/models`; `model` = published flow slug; Bearer API key | P0 |
| PB-2 | Per-flow MCP server (streamable HTTP) exposing `ask_<slug>`, `search_<kb>` and each tool | P0 |
| PB-3 | Platform MCP for coding agents, covering what a person does in the app: `list_flows`, `get_flow`, `update_flow`, `refine_flow` (with `focus` steps), `set_flow_active`, `get_integration_guide`, `create_flow_from_goal`, `get_build`, `answer_build_questions`, `list_step_types`, `list_knowledge_bases`, `add_documents`, `test_flow` (no side effects), `run_flow`, `get_run`, `list_runs`, `list_approvals`, `decide_approval`; local files a build attaches are uploaded first with `POST /mcp/files` (same API key) | P0 |
| PB-4 | Chat widget: one script tag; themable; streams answers with sources | P0 |
| PB-5 | Integration guide (`INTERLOCK.md`) generated per flow: endpoints, auth, schemas, examples in cURL, JS, Python, Rust | P0 |
| PB-6 | API keys: create, reveal once, revoke, last used | P0 |
| PB-7 | Shareable chat page per AI app | P1 |

### 5.7 Workspace, auth, connections (`il-server`, `il-connectors`)

| ID | Requirement | P |
|---|---|---|
| WS-1 | Sign in with Google or GitHub (name, email and picture only); sign out; delete your account and everything in its workspace | P0 |
| WS-2 | Removed 2026-10-30 (Q34): demo mode. Everyone signs in and works in their own workspace; templates are copied on use | — |
| WS-3 | Connections page: OAuth for Google (Gmail, Sheets, YouTube), Slack, Discord/Telegram bots, Bluesky app password, API keys for "Any API"; tokens encrypted at rest; never returned to the browser | P0 |
| WS-4 | Approvals inbox (desktop and mobile) | P0 mobile / P1 desktop |
| WS-5 | Usage: spend by day and by model role; caps editable | P1 |

### 5.8 Marketing site

| ID | Requirement | P |
|---|---|---|
| MK-1 | Landing page per design | P0 |
| MK-2 | Templates gallery with 10 templates (F-A, F-B, F-C + 7 more) | P0 |
| MK-3 | Recorded runs for F-A and F-C, shown with their templates | P0 |

## 6. Non-functional requirements

| Area | Requirement |
|---|---|
| Instant feel | UI feedback < 100 ms; API p95 < 50 ms excluding model and connector calls; SSE event latency < 200 ms |
| Lightweight | One server binary (target < 40 MB) with the web app embedded; cold start < 1 s; first screen < 150 KB JS+CSS gzipped |
| Modularity | Adding a node, connector, provider or preview renderer touches only its own files (section 9). CI enforces crate dependency direction |
| Reliability | Runs survive server restarts; Human waits persist; idempotent connector sends (dedupe key per node run) |
| Security | Secrets encrypted with AES-256-GCM under a master key from env; SSRF guard on all outbound HTTP (block private ranges, redirects re-checked); webhook tokens; API keys hashed; CORS restricted; demo isolation; no secrets in logs; prompt-injected content can never change a node's destination, recipient or credentials (those come only from settings/credentials, never from item data, unless explicitly mapped by the user) |
| Cost safety | Global daily spend cap from env; per-run and per-workspace caps |
| Observability | Structured JSON logs (tracing); `/healthz`; per-run cost and token accounting |
| Portability | Builds and runs natively on Windows (PowerShell) and Linux; deploys as one Docker image |
| Testability | Every module has unit tests that run offline with fake providers; CI runs fmt, clippy (deny warnings), tests, web build, type drift check, dependency check |

## 7. Architecture

### 7.1 Overview

```
Browser (SvelteKit SPA, embedded)            Coding agents / apps
   │  REST + SSE                               │  OpenAI API · MCP
   ▼                                           ▼
┌───────────────────────── il-server (axum, one binary) ─────────────────────────┐
│ auth · routes (auto-registered) · static web · scheduler · webhook ingress       │
├──────────────┬──────────────┬──────────────┬──────────────┬─────────────────────┤
│ il-architect │ il-engine    │ il-publish   │ il-knowledge │ il-connectors/oauth │
│ goal→flow    │ executor     │ OpenAI API   │ ingest       │ app nodes           │
│ ask→diff     │ events       │ MCP servers  │ hybrid search│                     │
│ test+fix     │ approvals    │ widget/guide │              │                     │
├──────────────┴──────┬───────┴──────────────┴──────┬───────┴─────────────────────┤
│ il-nodes (built-in) │ il-media (ffmpeg + gen)     │ il-ai (model gateway)       │
├─────────────────────┴─────────────────────────────┴─────────────────────────────┤
│ il-sdk: NodeExecutor/Trigger traits · NodeCtx · registries (inventory) · testkit │
├──────────────────────────────────────────────────────────────────────────────────┤
│ il-core: Flow · Node · Edge · Item · Manifest · Events · DTOs · expressions      │
├──────────────────────────────────────────────────────────────────────────────────┤
│ il-store: SQLite (rusqlite bundled) · sqlite-vec · FTS5 · files on disk          │
└──────────────────────────────────────────────────────────────────────────────────┘
External: Nebius Token Factory (Nemotron Ultra/Super/Nano, embeddings) · Tavily ·
          media generation providers · Google/Slack/Discord/Telegram/Bluesky/YouTube APIs
```

### 7.2 Crates and dependency direction (CI-enforced)

```
il-core  ←  il-sdk  ←  { il-nodes, il-connectors, il-media, il-knowledge, il-ai, il-store }
                    ←  il-engine  (depends on il-core + il-sdk traits only)
                    ←  il-architect, il-publish  (depend on il-core + il-sdk + traits; never on concrete crates)
il-server  →  everything (the only crate that wires concrete implementations together)
```

Rules:
- `il-core` has no IO and no async runtime dependency.
- `il-engine`, `il-architect`, `il-publish` talk to storage, models, files, knowledge, runs and each other **only through traits defined in `il-sdk`**: `Store`, `ModelGateway`, `FileStore`, `KnowledgeService`, `Notifier`, `MediaGen`, `Clock`, `FlowRunner` (start/resume/cancel runs, used by architect test runs and publish), `ArchitectService` (start builds, used by the server and the platform MCP) and `AgentTool` (a callable tool with name, JSON schema and invoke, provided by OpenAPI operations, knowledge search and nodes, consumed by the Agent node).
- Node crates (`il-nodes`, `il-connectors`, `il-media`) never import `il-store` or `il-server`. `il-knowledge` may use `il-store` for its own `kb_` tables; everything else reaches storage through the `Store` trait.
- `cargo xtask check-deps` fails CI on any violation.

| Crate | Owns | Must not |
|---|---|---|
| `il-core` | Domain types, JSON Schemas, event types, API DTOs, expression evaluation (minijinja), error types | Do IO |
| `il-sdk` | `NodeExecutor`, `TriggerNode`, `NodeCtx`, service traits, registries, manifest builder, `testkit` (fakes, harness) | Contain node logic |
| `il-store` | SQLite schema + migrations, repositories implementing `Store`, vector/FTS tables, `FileStore` on disk, secret encryption | Contain business logic |
| `il-ai` | `ModelGateway` impl: providers (Token Factory, fake), role → model mapping from config, JSON mode, tool calling, streaming, embeddings, transcription, cost metering, fallbacks | Hardcode model IDs outside config |
| `il-engine` | Executor, scheduler service, trigger supervisor, approvals, caps, event bus, replay | Know any specific node |
| `il-nodes` | Built-in nodes: `trigger/`, `ai/`, `knowledge/`, `logic/`, `data/`, `human/`, `output/` (one module per node) | Import store/server |
| `il-connectors` | OAuth broker + one module per app (`gmail/`, `sheets/`, `slack/`, `discord/`, `telegram/`, `bluesky/`, `youtube/`, `shopify/`, `social_dry_run/`, `openapi/`) | Share state between apps except via `NodeCtx` |
| `il-media` | ffmpeg operations, `MediaGen` trait and adapters (video, voice, image), recorded provider | Block the async runtime (use spawn_blocking / child processes) |
| `il-knowledge` | Parsers, chunker, Tavily client, ingestion pipeline, hybrid search, `KnowledgeService` impl | Call models except through `ModelGateway` |
| `il-architect` | Goal → flow compiler, catalog prompting, validation/repair, layout, connection check, test-and-fix, Ask diff, AI-app tests and guardrails | Hardcode node types (reads the registry) |
| `il-publish` | OpenAI-compatible API, MCP servers (rmcp), widget JS, integration guide, API keys | Execute flows except via the engine trait |
| `il-server` | axum app, auth (sign-in, sessions), route mounting (auto-registered), SSE, static assets (rust-embed), config loading, wiring | Contain domain logic |

### 7.3 Web app modules (`web/`)

SvelteKit (Svelte 5, runes) + adapter-static SPA + TypeScript strict + pnpm. Canvas uses `@xyflow/svelte`. No UI component framework; components follow the Claude Design system.

| Path | Owns |
|---|---|
| `src/lib/design/` | Tokens (`tokens.css`), base components (Button, Input, Pill, Tabs, Modal, Sheet, Toast…), icons |
| `src/lib/api/` | Typed client, SSE helper, **generated** types in `types/` (from Rust via ts-rs; never hand-edited), fixture mode |
| `src/lib/canvas/` | Svelte Flow wrapper, NodeCard, edges, groups, toolbar, minimap, palette, Replace menu, list view |
| `src/lib/forms/` | JSON-Schema form renderer with `x-widget` hints |
| `src/lib/previews/` | Output renderer registry; each renderer is one file, auto-discovered via `import.meta.glob` |
| `src/lib/panel/` | Node panel (Settings / Input / Output) |
| `src/lib/stores/` | App state (flows, runs, build session, auth) |
| `src/routes/` | One folder per screen: `/`, `/new`, `/new/[id]/building`, `/flows`, `/flows/[id]`, `/flows/[id]/runs/[run]`, `/flows/[id]/publish`, `/flows/[id]/chat`, `/templates`, `/connections`, `/approvals`, `/settings`, `/profile`, `/signin`, `/demo` |
| `widget/` | Chat widget source (separate tiny vanilla-TS bundle, no framework), built separately and served by `il-publish` at `/w/:slug.js` |

**Fixture mode**: `VITE_FIXTURES=1` serves all API calls from `web/fixtures/*.json` and replays recorded SSE streams, so the UI can be built and demoed before the backend exists.

### 7.4 Key technical decisions (do not re-litigate)

| Topic | Decision |
|---|---|
| Language/runtime | Rust (stable, edition 2024), tokio |
| HTTP | axum; tower-http for CORS, compression, tracing |
| Storage | SQLite via rusqlite (bundled) behind an async wrapper; sqlite-vec statically linked; FTS5; files on local disk under `data/files/` |
| Schemas and TS types | serde + schemars (JSON Schema for node settings) + ts-rs (TS types generated into `web/src/lib/api/types/`) |
| Expressions | minijinja, sandboxed, with helpers (`json`, `first`, `join`, `default`) |
| Registration | `inventory` crate: nodes, triggers, providers, routes and connectors self-register; no central lists to edit |
| Outbound HTTP | reqwest (rustls) through an SSRF-safe client in `il-sdk` |
| MCP | rmcp (official Rust SDK), streamable HTTP transport |
| Scheduler | tokio-cron-scheduler (timezone-aware) |
| Secrets | aes-gcm with `IL_MASTER_KEY` |
| Media | ffmpeg binary invoked as a child process; generation via HTTP providers behind `MediaGen` |
| Web | SvelteKit SPA, @xyflow/svelte, vitest, Playwright |
| Packaging | rust-embed for web assets; multi-stage Dockerfile; runtime image includes ffmpeg |
| Models | Roles in config, never in code: `architect` = Nemotron 3 Ultra, `agent` = Nemotron 3 Super, `fast` = Nemotron 3 Nano, `embed` = Qwen3 Embedding 8B; all via Token Factory's OpenAI-compatible API. Verify IDs against `GET /v1/models` at startup and log them |

## 8. Contracts (the seams every module plugs into)

Contracts live in `il-core` (Rust) and are exported to `contracts/` (JSON Schema) and `web/src/lib/api/types/` (TypeScript). **Additive changes** (new optional fields, new event types, new endpoints) may be made inside any task, with regenerated types. **Breaking changes** need a `contract:` PR (see AGENTS.md).

### 8.1 Flow

```jsonc
{
  "id": "flw_01J…", "workspaceId": "ws_…", "version": 3,
  "name": "Weekly product promos", "kind": "automation",        // "automation" | "app"
  "nodes": [{
    "id": "n_script", "type": "ai.write", "typeVersion": 1,
    "name": "Write video script",
    "position": { "x": 640, "y": 120 }, "group": "g_make",
    "settings": { "instructions": "…", "tone": "upbeat", "input": "{{ item.title }}" },
    "credentials": {},                                            // { "<credKey>": "cred_…" }
    "disabled": false, "notes": null
  }],
  "edges": [{ "id": "e1", "from": { "node": "n_products", "port": "out" }, "to": { "node": "n_script", "port": "in" } }],
  "groups": [{ "id": "g_make", "label": "Make the videos" }],
  "settings": { "timezone": "UTC", "spendCapUsdPerRun": 3.0 },
  "meta": { "createdBy": "architect", "goal": "Every Monday…" }
}
```

### 8.2 Items and files

Nodes exchange arrays of items: `Item { json: serde_json::Value, files: Map<String, FileRef> }`, where `FileRef { id, mime, bytes, name, previewUrl }` points to the FileStore. Each port carries `Vec<Item>`.

### 8.3 Node manifest and executor

```rust
pub struct NodeManifest {
    pub type_id: &'static str,          // "ai.write", "app.gmail.reply"
    pub version: u32,
    pub category: Category,             // Trigger | Ai | Knowledge | Media | Logic | Data | App | Human | Output
    pub title: &'static str,
    pub description: &'static str,      // also used by the architect
    pub icon: &'static str,             // icon key
    pub app: Option<&'static str>,      // logo slot key for app nodes
    pub inputs: Vec<PortSpec>,          // usually [in]; triggers have none
    pub outputs: Vec<PortSpec>,         // [out] or named branches; Switch derives from settings
    pub exec_mode: ExecMode,            // PerItem | Batch
    pub settings_schema: Schema,        // JSON Schema; x-widget hints: expression, credential, file, longtext, select, model-role
    pub credentials: Vec<CredentialSpec>,
    pub side_effect: SideEffect,        // None | External (sends, posts, writes outside Interlock)
    pub status: NodeStatus,             // Ready | DryRun | Experimental
    pub cost_hint: CostHint,
    pub examples: Vec<NodeExample>,     // short examples the architect sees
}

#[async_trait]
pub trait NodeExecutor: Send + Sync + 'static {
    fn manifest(&self) -> &NodeManifest;
    fn validate(&self, settings: &Value) -> Result<(), Vec<ValidationIssue>> { default_schema_validate(self, settings) }
    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError>;
}

#[async_trait]
pub trait TriggerNode: NodeExecutor {
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError>;   // schedule, poll, webhook bind
}

inventory::collect!(NodeFactory);
// In each node module:
inventory::submit! { NodeFactory::new(|| Box::new(AiWrite::default())) }
```

`NodeCtx` gives: rendered settings, `credential(key)`, `models()` (ModelGateway), `files()`, `knowledge()`, `http()` (SSRF-safe), `emit_progress(pct, msg)`, `request_approval(spec)`, `cancel_token()`, `run_info()`, `dedupe_key()`.

### 8.4 Events (runs and builds)

All events: `{ seq, ts, runId | buildId, type, nodeId?, data }`, ordered per stream, persisted, served over SSE at `/api/runs/:id/events?after=<seq>` and `/api/builds/:id/events?after=<seq>`.

Run events: `run.started`, `node.started`, `node.progress`, `node.items` (preview of items, max 3), `node.succeeded` {count, ms, costUsd}, `node.failed` {error, retryable}, `node.waiting` {approvalId}, `approval.decided`, `run.cap_reached`, `run.succeeded`, `run.failed`, `run.cancelled`.

Build events: `build.stage` {stage, status}, `build.log` {text}, `build.question` {questions}, `build.flow_patch` {jsonPatch} (nodes/edges appear incrementally), `build.connection_needed` {app}, `build.test_result` {passed, failed}, `build.fix` {description}, `build.ready` {flowId}, `build.failed`.

Unknown event types must be ignored by the UI (forward compatible).

### 8.5 HTTP API (internal, `/api`)

`GET /api/nodes` (catalog) · `GET|POST /api/flows` · `GET|PUT|DELETE /api/flows/:id` · `GET /api/flows/:id/versions` · `POST /api/flows/:id/activate|deactivate` · `POST /api/flows/:id/test` · `POST /api/flows/:id/ask` → diff · `POST /api/flows/:id/nodes/:nodeId/run` · `GET /api/flows/:id/runs` · `GET /api/runs/:id` · `GET /api/runs/:id/events` (SSE) · `POST /api/approvals/:id/decide` · `GET /api/approvals` · `POST /api/builds` (goal + attachments) · `GET /api/builds/:id/events` (SSE) · `POST /api/builds/:id/answers` · `POST /api/files` (multipart) · `GET /api/files/:id` · `GET|POST /api/knowledge` · `GET /api/connections` · `GET /api/oauth/:app/start` · `GET /api/oauth/:app/callback` · `GET /api/templates` · `POST /api/templates/:id/use` · `GET|DELETE /api/me` · `GET /api/auth/providers` · `GET /api/auth/:provider/start|callback` · `POST /api/auth/sign-out` · `POST /api/demo/session`.

Public: `POST /hooks/:flowId/:token` · `POST /v1/chat/completions` · `GET /v1/models` · `/mcp` (platform) · `/mcp/:slug` (per flow) · `GET /w/:slug.js` (widget) · `GET /healthz`.

All errors: `{ "error": { "code": "snake_case", "message": "Human sentence.", "details": {} } }`.

### 8.6 Model gateway

```rust
#[async_trait]
pub trait ModelGateway: Send + Sync {
    async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError>;        // tools, json_schema, temperature
    async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError>;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError>;
    async fn transcribe(&self, file: &FileRef) -> Result<Transcript, ModelError>;
}
```

Roles: `Architect`, `Agent`, `Fast`. Every call records tokens and cost against the current run/build. If native tool calling or JSON-schema output is unavailable for a model, the gateway falls back to a JSON-in-prompt protocol with validation and repair, transparently to callers.

## 9. Extension guide (the modularity promise)

| To add… | Touch only | Nothing else changes because… |
|---|---|---|
| A node | One module in `il-nodes` or `il-connectors` (+ optional preview renderer) | It self-registers via `inventory`; the palette, settings form, architect catalog and docs are generated from its manifest |
| An app connector | `il-connectors/<app>/` (nodes + OAuth config entry) | OAuth broker reads provider configs from the registry |
| A model provider | `il-ai/providers/<name>.rs` | Providers self-register; roles map to providers in config |
| A media generator | `il-media/gen/<name>.rs` | Same pattern via `MediaGen` registry |
| An output preview | `web/src/lib/previews/<kind>.svelte` | Auto-discovered by `import.meta.glob`; matched on item mime/kind |
| A screen | `web/src/routes/<screen>/` | File-system routing |
| An API route group | A module exposing `routes()` + `inventory::submit!(RouteFactory…)` | Server mounts all registered route factories |
| A template | `fixtures/templates/<slug>.json` | Loaded at startup and validated against the catalog |
| A DB table | A new timestamped migration `migrations/<YYYYMMDDHHMM>_<module>_<desc>.sql`, tables prefixed by module | Migrations run in order; prefixes prevent clashes |

## 10. Data model (SQLite)

`ws_workspaces`, `ws_users`, `ws_members`, `ws_api_keys` · `fl_flows`, `fl_versions` · `rn_runs`, `rn_events`, `rn_node_runs`, `rn_approvals` · `tr_schedules`, `tr_webhooks`, `tr_poll_cursors` · `cn_credentials` (encrypted), `cn_oauth_states` · `fs_files` · `kb_bases`, `kb_sources`, `kb_chunks`, `kb_chunks_vec` (vec0), `kb_chunks_fts` (fts5) · `bd_builds`, `bd_events` · `pb_endpoints` · `us_ledger` (cost per call).

## 11. Configuration

`config/interlock.toml` with env overrides (`IL_…`). Sections: `[server]`, `[auth.github]`, `[models]` (base_url, roles → model IDs, prices), `[providers.media]`, `[providers.transcription]`, `[tavily]`, `[caps]` (global_daily_usd, demo limits), `[storage]` (path), `[security]` (master key from env only). Secrets only from env: `NEBIUS_API_KEY`, `TAVILY_API_KEY`, `IL_MASTER_KEY`, `GITHUB_CLIENT_SECRET`, `GOOGLE_CLIENT_SECRET`, `SLACK_CLIENT_SECRET`, media provider keys. Tests use `IL_PROVIDERS=fake` and need no network.

## 12. Deployment

- Multi-stage Dockerfile: build web (pnpm) → build Rust (release, embed web) → runtime (debian-slim + ffmpeg + ca-certificates), non-root, `/data` volume.
- Host on Nebius (Serverless Endpoint CPU or a small VM) with a persistent volume and HTTPS. Confirm the endpoint can be public without a token for judges; if not, use a VM behind a TLS proxy.
- `/healthz` for health checks; global spend cap on.

## 13. Hackathon compliance checklist

The team stopped pursuing the hackathon submission on 30 Oct 2026; Interlock continues as a product. Kept for the record.

- [ ] Runtime calls to Nebius Token Factory; NVIDIA Nemotron Ultra, Super and Nano used by role (documented in README with where each is used)
- [ ] Public GitHub repo with an Apache-2.0 LICENSE detectable in the About section; README with setup and run instructions
- [ ] Working demo URL, free and unrestricted for judges until 15 Dec 2026 (demo mode, no sign-in)
- [ ] Demo video < 3 min, public on YouTube, no copyrighted music or third-party trademarks without permission
- [ ] Track: Best Apps and Agents
- [ ] Feedback on Token Factory, AI Cloud and NVIDIA models in the submission
- [ ] Note that the project builds on ideas from the team's earlier project Tilo, with a new codebase, design and feature set
- [ ] Functional runtime call to the Tavily API (website ingestion) for Best Use of Tavily
- [ ] All materials in English

## 14. Milestones and definition of done

| When (IST) | Milestone | Done when |
|---|---|---|
| Tue 27, midday | **M0 Contracts & skeleton** (T00) | Workspace compiles on Windows and Linux; all crates and traits exist with fakes; web scaffold in fixture mode; CI green; AGENTS.md and WORKBOARD.md in repo |
| Wed 28, night | **M1 Core loop** | A hand-written flow runs end to end through the engine with real Nemotron calls; editor renders and edits it; runs stream live |
| Thu 29, evening | **M2 Feature complete (P0)** | Architect builds F-A, F-B, F-C from their sentences; knowledge, publish, connectors and media work; first deploy on Nebius |
| Fri 30, 6 PM | **M3 Ship** | Demo mode seeded; E2E smoke green; README, video and Devpost submitted (hard deadline 10:30 PM IST) |

Global definition of done for any task: acceptance criteria met; unit tests added and passing offline; `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `pnpm check`, `pnpm test`, `cargo xtask check-deps` and type drift check pass; docs updated where behaviour changed; WORKBOARD updated.

## 15. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Nemotron tool calling / JSON schema output behaves differently than expected | Gateway fallback to JSON-in-prompt + validation/repair; architect uses strict schemas and few-shot examples from manifests |
| Social platforms block posting | Dry-run nodes with previews; real posting via YouTube (private), Bluesky, Discord, Telegram |
| Video/voice generation cost or latency | Not live-tested: adapters ship, the recorded provider is the default everywhere |
| Integration crunch on Friday | Contracts first; fixture mode for UI; fakes for every service; integrator task starts Thursday |
| Merge conflicts between agents | Owned paths per task; self-registration (no central lists); timestamped migrations; small PRs; rebase before merge |
| Windows build issues (C deps) | Bundled SQLite and statically linked sqlite-vec; MSVC toolchain documented; CI on both Windows and Linux |
| Public endpoint auth on Nebius Serverless | Verify early (task R02); fall back to a VM |
| Name collision ("Interlock" is also used by a ransomware group) | Check before publishing the landing page; rename is a config + asset change |

## 16. Glossary

Flow · Node · Edge · Port · Item · Run · Build (architect session) · Trigger · Approval · Connection (stored credential) · Knowledge base · Publish · Catalog (all registered node manifests) · Role (architect/agent/fast model role).
