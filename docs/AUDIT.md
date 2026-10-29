# Capability audit — 29 Oct 2026

What Interlock really does today, end to end, measured against the PRD's P0 scope. This is a snapshot of `main` at `75a1834`, taken the day before the deadline. Everything ran offline in fake mode (`IL_PROVIDERS=fake`), on Windows with ffmpeg installed.

**Bottom line.** The engine, the editor, the templates, test runs, approvals, webhooks, schedules and publishing (OpenAI API, both MCP servers, the widget) really work end to end. The biggest P0 hole is **knowledge**: the server never wires the real `il-knowledge` service, so every knowledge base, search, document parse and MCP `add_documents` call goes to the testkit fake. F-C can't answer from documents. After that:

- the Building screen never marks steps ready;
- three UI calls broke against the real server (fixed in #56);
- API keys need GitHub sign-in, so `/v1` and `/mcp` are unusable on a fresh deploy and in the demo;
- demo caps aren't enforced;
- Google tokens never refresh, so Gmail (F-B) stops after an hour;
- a few cross-workspace holes.

Everything real-model, real-connector or deploy-related is still untested because there are no keys yet. That work is already on the board as Q01, R02 and R03.

## How this was tested

| Step | Result |
|---|---|
| `cargo xtask check` | fmt, clippy, check-deps and type/catalog drift pass. 323 of 324 Rust tests pass. **1 failure**: `every_template_test_runs_end_to_end_in_fake_mode` reported `tpl_promos: running` because the test polls for only 5 s, and with ffmpeg present the promos media steps take about 13 s. CI has no ffmpeg, so it skips that template. Fixed in #59. Web: `pnpm check` 0 errors, vitest 73/73 |
| Real server | `target/debug/il-server` in fake mode with the example config, a scratch data dir and a test `IL_MASTER_KEY`. Driven with Python over HTTP: every route group, all 10 templates (use → test → poll), 4 architect builds, approvals, cancel, webhook, schedule, publishing, MCP and widget. Restarted mid-approval to check recovery |
| Browser | Built web app served by the binary. Clicked through landing, demo, flows, editor, Building, run view, approvals, publish, playground, connections and settings |
| Static | Every node executor in `docs/NODES.md`, every connector operation, every web route's API calls |

To test keys, `/v1` and `/mcp` without a GitHub app, a signed-in session row was written straight into the scratch SQLite store (`ws:session:*`). See bug 4.

## Capability matrix

Status: **real** (works end to end in fake mode, with real code paths) · **partial** (works with a stated gap) · **stub** (placeholder) · **missing** · **untested** (real code that needs keys or accounts we don't have).

### Create and architect (PRD §5.1)

| Feature | PRD | Status | Evidence |
|---|---|---|---|
| Goal + attachments accepted | AR-1 | partial | `POST /api/builds` takes `attachments`, but they only become text in the brief (`il-architect/src/compile/build.rs:307-330`). Knowledge files and links are **never ingested** into a knowledge base. Audio marked Instructions is transcribed |
| Clarifying questions | AR-2 (P1) | real | `build.question` → `POST /api/builds/:id/answers` (`build.rs:343`). The UI handling was broken until #56 |
| Goal → valid flow from the catalog | AR-3 | real (offline: template match) | Fake mode picks the seeded template whose name and goal share the most words (`build.rs:196`). F-A, F-B and F-C each got the right template. "Webhook lead → Discord" got *Meeting notes to Slack*, failed its test and was not fixed. The real compile path only has golden tests (`il-architect/tests/golden.rs`) and is not checked against Nemotron (Q01) |
| Prompts, mappings, groups, layout | AR-4 | real | Templates and golden flows carry prompts, expressions and groups. `layout/` |
| Connections detected | AR-5 | partial | `build.connection_needed` per app (`build.rs:378`). It also flags Instagram, TikTok and LinkedIn, whose dry-run nodes never read a credential |
| Test and fix (2 rounds) | AR-6 | real (offline untested) | `test_and_fix` (`build.rs:233`). In fake mode `revise` can't repair, so failures are only logged |
| Build events stream | AR-7 | partial | `build.stage/log/flow_patch/cost/test_result/ready` are emitted. **`build.node` and the `build.tests` node are never emitted**, so in the browser every step stays "Waiting" after "Ready" |
| Ask Interlock diff | AR-8 | untested offline | `POST /api/flows/:id/ask` → 502 in fake mode: "the flow still had problems after 3 repairs: … needs a trigger step". It needs a real model (Q01) |
| Guardrails + test suite for AI apps | AR-9 | partial | `appkit::suite` and `missing_guardrails` exist. The F-C template uses `ai.decide` as its guard, not `ai.guardrails` |
| Validate + repair loop | AR-10 | real | `validate/`, repair in `compile/` (golden tests) |

### Editor (PRD §5.2)

| Feature | PRD | Status | Evidence |
|---|---|---|---|
| Canvas: minimap, zoom, tidy, undo/redo, list view | ED-1 | real | Browser: editor at `/flows/:id` |
| Palette from the catalog | ED-2 | real | Approval was tagged "Soon": its manifest kept the default `stub` status (fixed in #58) |
| Schema-driven node panel | ED-3 | real | `lib/forms/`, `lib/panel/` |
| Field mapper chips | ED-4 | real | W03 |
| Output previews | ED-5 | real | Promos run shows clips and files from `/api/files/:id` |
| Replace with… | ED-6 | real | W02 |
| Run a single step | ED-7 (P1) | **stub** | `POST /api/flows/:id/nodes/:nodeId/run` → 501 "arrives with task W03 / ED-7" (`il-server/src/api/flows.rs:196`). The button is shown in the panel and fails with that message |
| Versions | ED-8 (P1) | partial | `GET /api/flows/:id/versions` works. There is no UI and no restore |
| Ask bar + diff view | ED-9 | real UI, model untested | See AR-8 |

### Engine and runs (PRD §5.3)

| Feature | PRD | Status | Evidence |
|---|---|---|---|
| Graph: per-item/batch, branches, merges, concurrency | EN-1 | real (one gap) | All 10 templates run. Parallel per-item branches don't pair items when they join (already Q08) |
| Retries, timeouts, cancellation | EN-2 | real | `POST /api/runs/:id/cancel` → 202, run `cancelled`. There is no cancel button in the UI |
| Pause at Human, resume, survives restart | EN-3 | real | Killed the server with `tpl_translate` waiting, restarted, approved, and the run `succeeded` (Telegram step ran as a test preview) |
| Ordered events + SSE | EN-4 | real | `/api/runs/:id/events`, `/api/builds/:id/events`. The UI retries forever on 404 (no error state) |
| Triggers | EN-5 | real | Webhook: activate → `GET /api/flows/:id/hooks` → `POST /hooks/…` → 202 and the run succeeded. A bad token gets 404. Schedule activates, and triggers are restored on restart (`restored=1`). Chat, API and MCP go through `invoke`. Gmail/Shopify polling uses `il_sdk::poll` (real Shopify polling isn't implemented) |
| Spend caps | EN-6 | partial | Run, flow, workspace and global caps are checked from `us_ledger` (`il-engine/src/caps/mod.rs:39`). **Demo caps (`demo_run_usd`, `demo_models`, `demo_media`) are read by nothing**. Architect build spend is never written to the ledger, so builds are uncapped |
| Run history with node I/O | EN-7 | real | `GET /api/runs/:id` → `{run, nodeRuns}` with inputs, outputs and files |
| Replay recorded runs | EN-8 | real | Demo promos and Larkspur show "Ran yesterday" from the recorded runs |

### Knowledge (PRD §5.4)

| Feature | PRD | Status | Evidence |
|---|---|---|---|
| Ingest PDF/DOCX/MD/TXT/CSV | KN-1 | **stub at runtime** | Parsers are real (`il-knowledge/src/parse/`), but **the server keeps `FakeKnowledge`**: `build_state_persistent` starts from `FakeServices::default().services()` and never replaces `knowledge` (`il-server/src/wiring.rs:87-104`). `Knowledge::new` is called only in il-knowledge's own tests |
| Tavily crawl | KN-2 | **stub at runtime** | Same. `Tavily` is never constructed by the server |
| Transcribe audio as knowledge or instructions | KN-3 | partial | Build-time transcription of Instructions audio works through the gateway. Transcripts never reach a knowledge base |
| Chunk + embed | KN-4 | stub at runtime | Same as KN-1 |
| Hybrid search with sources | KN-5 | **stub at runtime** | Live: the agent's `sources` were always `[]`. Per-flow MCP `search_n_kb` → "knowledge base not found". `knowledge.base` also creates a new base on every run (`il-nodes/src/knowledge/base.rs:102`) |
| Add documents (UI/API/MCP) | KN-6 (P1) | stub | `GET` and `POST /api/knowledge` → 501 "arrives with task K01" (`il-server/src/api/misc.rs:84`). MCP `add_documents` → "knowledge base not found" (the fake's `owns_base` returns false) |
| Parse document node | §5.5 | stub at runtime | Returns `{"text":"Parsed text of <id>"}`. Once wired: the `format` override is ignored, and the real output has `sections` but no `text`, which the node's test and templates expect |

### Nodes, connectors, media (PRD §5.5)

All 60 catalog nodes have real executors. None returns `NotImplemented`. The node-by-node table and the connector-by-connector table are in the appendix.

| Area | Status | Evidence |
|---|---|---|
| Triggers (8) | real | `trigger.mcp_call` ignores `toolName`/`description`/`inputSchema`. The webhook ignores `method` and accepts POST and GET only |
| AI nodes (13) | real | Use roles through `ModelGateway`. `ai.describe_image` sends images to the `agent` role (Nemotron Super may not accept images). `ai.transcribe` ignores `language`. The `ai.guardrails` hand-off only sets a flag |
| Knowledge nodes (3) | stub at runtime | See §5.4 |
| Media ops (5 + convert) | real | ffmpeg child processes. Promos produced merged, resized, captioned clips. Hard 120 s ffmpeg timeout. Each op writes its own file key, so chains need explicit mapping |
| Media generation (3) | real (recorded) | `recorded` is the default everywhere. The Runway adapter is untested, and its polling ignores cancel |
| Logic and data (11) | real | `data.http` replaces `item.json` with the response |
| Human (2) | real | Approval's status was wrong (#58). A per-item flow creates one approval per item: F-A asks three times for three products |
| Output (2) | partial | `output.notify` declares no credentials, and `inapp`/`email` go to the testkit `FakeNotifier`, which reports success and delivers nothing |
| Gmail, Sheets, YouTube | real HTTP, untested live | wiremock tests on the real paths. **No token refresh**: nodes read `access_token` and never check `expires_at`, so calls fail about an hour after connecting |
| Slack, Discord, Telegram, Bluesky | real HTTP, untested live | No dedupe key (a retry can double-post). Slack uses a saved bot token. There is no Slack OAuth provider. The Slack approval message is never updated after a click |
| Shopify | real (sandbox) | Real Admin API `get_products` exists. Real `new_product` polling is not implemented |
| Instagram, TikTok, LinkedIn, X | dry-run by design | Preview only. In a test run the engine's `testRun` preview replaces the `dryRun` preview |
| Use any API (OpenAPI) | real | Node plus agent tools (R05). **Agent tools call real APIs during test runs**: the R08 guarantee covers external nodes only (`il-connectors/src/openapi/tool.rs:103`) |

### Publish and connect (PRD §5.6)

| Feature | PRD | Status | Evidence |
|---|---|---|---|
| `/v1/chat/completions` + `/v1/models` | PB-1 | partial | Non-stream works with a key. `stream: true` sends the whole answer as **one chunk after the run ends** (`il-publish/src/openai/mod.rs:158`). Content-part messages (`content: [{type:"text"}]`, valid in the OpenAI API) → 400 "a user text message is required" (`openai/mod.rs:118`). Unpublished apps disappear from `/v1/models` |
| Per-flow MCP | PB-2 | partial | `initialize`, `tools/list` → `ask_larkspur-support-agent` and `search_n_kb`. `ask` works. `search_*` fails (fake knowledge). The KB tool is named after the node id, not the base |
| Platform MCP | PB-3 | real | `list_flows`, `get_integration_guide`, `create_flow_from_goal` and `run_flow` work. `add_documents` fails (fake knowledge). The Publish page's `claude mcp add … /mcp` line has **no Authorization header**, so it gets 401 (`web/src/routes/flows/[id]/publish/AgentCard.svelte:28`) |
| Chat widget | PB-4 | real | `GET /w/<slug>.js` (10.9 KB). `POST /w/<slug>/chat` answers without a key (by design), with rate limiting |
| Integration guide | PB-5 | real API, UI broken | `GET /api/flows/:id/guide` returns markdown. The Publish page expected JSON and always showed the fallback (fixed in #56) |
| API keys | PB-6 | partial | Create, reveal once, list and revoke work for a signed-in session. **Without GitHub sign-in a key can't be created**: 401 "Sign in first." even in the local workspace (`il-publish/src/keys/routes.rs:32`). Demo → 403 (`:41`). So on a fresh deploy nobody can use `/v1` or `/mcp` |
| Shareable chat page | PB-7 (P1) | missing | — |

### Workspace, auth, connections (PRD §5.7)

| Feature | PRD | Status | Evidence |
|---|---|---|---|
| GitHub sign-in | WS-1 | untested | `/api/auth/github/start` → 503 `github_unconfigured` with no app (waiting on the owner) |
| Demo mode | WS-2 | partial | `POST /api/demo/session` gives an ephemeral workspace seeded with 10 templates and the recorded runs. Keys and connections → 403. **Not done**: demo caps and model limits (see EN-6), daily reset (Q04). Settings tells demo users "small caps, the smaller models", which is not true yet |
| Connections | WS-3 | partial | API-key connections save encrypted (`POST /api/connections` → 201 with `IL_MASTER_KEY`). Without `IL_MASTER_KEY`, saving fails. Google OAuth start → 503 until configured. **Only Google has an OAuth provider**; Slack OAuth (PRD) is missing. No refresh (see above) |
| Approvals inbox | WS-4 | real (UI fixed in #56) | Approve/Reject applied server-side (202), but the UI threw on the empty body, so the card stayed with an error toast |
| Usage and caps page | WS-5 (P1) | partial | `GET /api/usage` works and shows `totalUsd: -0.0` with no spend. Caps are read-only (U02) |

### Marketing and demo (PRD §5.8), NFRs

| Feature | PRD | Status | Evidence |
|---|---|---|---|
| Landing | MK-1 | real | Hero replays the bundled build |
| 10 templates | MK-2 | real | All 10 test-run in fake mode: 8 succeed, and promos and translate stop at their approval as designed |
| Recorded runs F-A, F-C | MK-3 | real | Seeded into demo workspaces |
| One binary with embedded web | §6 | real | It must be rebuilt after `pnpm -C web build`. Without the build, `/` says "The web app is not built" |
| Runs survive restarts | §6 | real | See EN-3 |
| Idempotent sends | §6 | partial | Only Gmail, Sheets and YouTube use `dedupe_key` |
| Demo isolation, no cross-workspace access | §6 | **bugs** | See bugs 5 and 6 |

## Bugs found

Fixed in their own PRs (each under 30 lines):

1. **Approve/Reject and build answers fail in the UI** (fixed in #56). Repro: `/demo` → open *Weekly product promos* → Test run → `/approvals` → Approve. The network shows `POST …/decide → 202` with an empty body, `web/src/lib/api/client.ts` calls `res.json()` and throws, and the card stays until a reload. The same happens with `POST /api/builds/:id/answers`.
2. **The integration guide never shows** (fixed in #56). Repro: publish *Larkspur support agent* → Integration pack says "appears once published". The server sends `text/markdown`, and the client expected `{markdown}`.
3. **Approval node listed as `stub`** (fixed in #58). Repro: `GET /api/nodes` → `human.approval` has `status: "stub"`, so the palette shows "Soon".
4. **Template test fails locally with ffmpeg** (fixed in #59). Repro: `cargo xtask check` on Windows with ffmpeg → `tpl_promos: running null`.

Filed as tasks (see the next section):

5. **Files are readable across workspaces.** `GET /api/files/:id` ignores the caller's workspace (`il-server/src/api/misc.rs:77`). Repro: upload in a demo session, then `curl /api/files/<id>` with no cookie → 200 with the content. → Q10
6. **Build streams and answers aren't scoped.** `GET /api/builds/:id/events` and `POST /api/builds/:id/answers` take no workspace (`il-server/src/api/builds.rs:57,72`), so anyone with a build id can read the goal, flow and logs. OAuth start/refresh with no cookie act on `ws_local` (`il-connectors/src/oauth/routes.rs:79`). → Q10
7. **Knowledge is fake in production** (`wiring.rs:87`). Repro: test-run *Larkspur support agent* → `sources: []`. Per-flow MCP `search_n_kb` → "knowledge base not found". → K03
8. **Building screen never finishes visually.** Repro: `/new` → F-C goal → Build it. After "Ready", all eight steps still say "Waiting". → A03
9. **No API key without GitHub.** Repro: `POST /api/keys` with no session → 401 "Sign in first."; in demo → 403. → P02
10. **The platform MCP install line lacks auth.** `claude mcp add --transport http interlock …/mcp` → 401 on use. → P02
11. **OpenAI content parts rejected.** `messages:[{role:"user",content:[{type:"text",text:"Hi"}]}]` → 400. → P02
12. **Agent API tools run for real in test runs** (`openapi/tool.rs:103`). This breaks R08's promise for F-C's invoice tools and any `allowRisky` operation. → C07
13. **Google tokens never refresh.** No caller of `Broker::refresh` except a manual route, and nodes never check `expires_at`. F-B stops about an hour after connecting. The comment at `il-server/src/api/connections.rs:20` says otherwise. → C07
14. **Demo caps unenforced.** `demo_run_usd`, `demo_models` and `demo_media` are read nowhere. Build spend isn't in `us_ledger`. → B10
15. **`output.notify` "succeeds" without sending** for `inapp`/`email` (testkit `FakeNotifier`), and Slack/Discord/Telegram credentials can't be bound (no credential specs). → N02
16. **Run view shows "Running"** while the run is `waiting` at an approval, and the mobile step list's `onselect` is a no-op (`routes/flows/[id]/runs/[run]/+page.svelte:78`). → W08
17. **`ws_local` session-less single-user mode** means an unauthenticated visitor to a public deploy acts on the local workspace: flows, connections, approvals. This is fine for self-hosting, but the public deploy must require a session. → Q10, to confirm with Q03.

Smaller notes (no task): `usage.totalUsd` prints `-0.0`. Stale doc comments say "returns `NotImplemented` until task C01/K02" on 7 implemented connector files. `il-core/src/manifest.rs:127` says `Stub` means NotImplemented. `[providers.transcription]` in the example config is unused. `/dev/components` ships in production builds. The WORKBOARD known issue "No `ai.guardrails` node" is out of date (N01 done).

## Ranked gaps against P0

| Rank | Gap | Why it matters | Task |
|---|---|---|---|
| 1 | Real knowledge service not wired | F-C (hero flow 3), KN-1…KN-5, PB-2 search and the MCP `add_documents` tool are all fake | **K03** (new) |
| 2 | Build attachments never become knowledge | AR-1, the F-C acceptance ("knowledge base built", "voice memo → instructions") | **K04** (new) |
| 3 | Real models never exercised | Architect compile, Ask diff, fixes and every AI node only work against the fake | Q01 (owner keys) |
| 4 | API keys need GitHub; MCP install line lacks auth; demo can't try the API | PB-1, PB-2, PB-3, PB-6 are unusable on a fresh deploy | **P02** (new) |
| 5 | Workspace isolation holes | Public demo: other people's files and builds are readable | **Q10** (new), then Q03 |
| 6 | Google token refresh, send dedupe, agent tools in test runs | F-B dies after an hour. Double posts. Test runs can hit real APIs | **C07** (new) |
| 7 | Building screen steps never turn ready | AR-7: the live "watch it build" moment, the demo's centrepiece | **A03** (new) |
| 8 | Demo caps unenforced, builds uncapped | WS-2 and cost safety on a public URL | **B10** (new) |
| 9 | Deploy + live connectors | Hackathon requirement, F-A/F-B real sends | R02, R03 (owner) |
| 10 | Demo daily reset | WS-2 promise shown in the UI | Q04 |
| 11 | Notify channels, ignored node settings | `output.notify` silently does nothing for in-app/email | **N02** (new, P1) |
| 12 | UI state gaps: Run this step (ED-7), cancel, delete flow, loading and SSE error states | Polish. ED-7 is P1 | **W08** (new, P1) |
| 13 | True token streaming on `/v1` and the widget | PB-1 says streaming; today it is one chunk | in P02 (stretch) |

Already on the board and confirmed by this audit: Q01, Q03, Q04, Q05 (claimed), Q06, Q08, R02, R03, U02.

## Appendix A — nodes

| Type | Executor | Verdict | Notes |
|---|---|---|---|
| trigger.manual / chat / api_call | `il-nodes/src/trigger/*.rs` | real | Entry through Test or `TriggerSupervisor::invoke`. `suggestedQuestions` is only used by the UI |
| trigger.mcp_call | `trigger/mcp_call.rs:57` | partial | `toolName`, `description` and `inputSchema` ignored (`il-publish/src/mcp/flow.rs:48`) |
| trigger.schedule | `trigger/schedule.rs:92` | real | Cron + tz loop. Missed fires are not caught up |
| trigger.webhook | `trigger/webhook.rs:94` | real | `method` ignored. Query and headers are not passed |
| ai.write / summarise / translate / classify / decide / extract | `il-nodes/src/ai/*.rs` | real | Roles through the gateway. JSON-schema output for classify, decide and extract |
| ai.agent | `ai/agent.rs:90` | real | Tool loop, `maxSteps` up to 20. KB search is fake at runtime. OpenAPI tools from the flow (R05) |
| ai.guardrails | `ai/guardrails.rs:47` | real | `handoff` is a flag only |
| ai.describe_image | `ai/describe_image.rs:71` | real | Agent role. Vision support of the configured model unknown |
| ai.transcribe | `ai/transcribe.rs:66` | real | `language` ignored |
| ai.generate_image / video / voice | `il-media/src/gen/*.rs` | real (recorded) | Runway adapter untested |
| knowledge.base / search / add | `il-nodes/src/knowledge/*.rs` | real code, **fake at runtime** | New base per run. `add` is text only |
| data.http / set_fields / template / openapi | `il-nodes/src/data/*.rs`, `il-connectors/src/openapi/call.rs` | real | — |
| data.parse_document | `data/parse_document.rs:63` | real code, **fake at runtime** | `format` ignored. Output shape mismatch |
| logic.if / switch / loop / merge / wait / stop | `il-nodes/src/logic/*.rs` | real | `wait` is in-process (not durable across restarts) |
| human.approval / ask | `il-nodes/src/human/*.rs` | real | Status fixed in #58 |
| output.respond | `output/respond.rs:62` | real | No streaming |
| output.notify | `output/notify.rs:72` | partial | See bug 15 |
| media.cut_clips / resize / captions / merge_av / thumbnail / convert | `il-media/src/ops/*.rs` | real | ffmpeg; one shared real-media test that skips without ffmpeg |
| app.* (14) | `il-connectors/src/<app>/` | real HTTP / dry-run | See Appendix B |

## Appendix B — connectors

| Connector | Operations | Real HTTP | Tests (wiremock on real paths) | Gaps |
|---|---|---|---|---|
| oauth | start, callback, manual refresh | Google token endpoint | 3 | Google only. No auto-refresh. Google-specific params (`access_type=offline`, requires `refresh_token`) |
| gmail | new_email (poll), send, reply, label | gmail.googleapis.com | 5 | No refresh. Dedupe ✓ |
| sheets | append_row | sheets.googleapis.com | 2 | Loose path matcher in the test |
| youtube | upload (private, resumable) | googleapis upload | 2 | Dedupe ✓ |
| slack | send, approval, `/api/slack/actions` callback, `MessagingNotifier` | chat.postMessage | 3 | Only the send failure path is tested. Notifier has 0 tests. No dedupe. The message isn't updated after a decision |
| discord | send (webhook or bot, attachments) | discord.com | 3 | No dedupe. The host check is off under `cfg(test)` |
| telegram | send (text/photo/video/document) | api.telegram.org | 2 | No dedupe |
| bluesky | post (image/video) | bsky.social XRPC | 2 | No dedupe. New session per post |
| shopify | get_products (sandbox or Admin GraphQL), new_product (sandbox) | myshopify.com GraphQL | 3 | Real `new_product` polling missing. Real `get_products` runs in test runs |
| social_dry_run | instagram, tiktok, linkedin, x | none (by design) | 3 | — |
| openapi | node, spec import (3.0/3.1, JSON/YAML, cURL), agent tools | spec's `servers[0]` | 8 | Agent tools ignore test runs. Relative server URLs fail. cURL import drops `-H` values |

Another session is working on connectors and model providers in parallel (B09, C05). This audit only reports on `il-connectors` and `il-ai`; it changes neither.
