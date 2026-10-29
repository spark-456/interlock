# WORKBOARD — Interlock

How to use this board: see `AGENTS.md` §3. Take the **first `ready` task from the top** whose hard deps are all `done`, claim it by editing its row, and push. **After T00 merges, every other task's only hard dependency is T00** (except A02, R02 and R03), so any free agent can take any task; soft deps are covered by fakes and fixtures.

Status values: `ready` · `claimed` · `review` · `done` · `blocked`
Owner names: `claude-a`, `claude-b`, `codex-1`, `codex-2`, `human` (add more as needed)

## Index (priority order — top first)

| # | ID | Task | Prio | Hard deps | Status | Owner | Branch | Updated (UTC) | PR / Notes |
|---|---|---|---|---|---|---|---|---|---|
| 1 | T00 | Contracts & skeleton | P0 | — | done | claude-a | t/T00-skeleton | 2026-10-27 05:15 | Merged locally (no remote yet), commit 90af90f. Read its commit message for notes to later tasks |
| 2 | B03 | Engine: executor, events, approvals, caps | P0 | T00 | done | claude-a | t/B03-engine | 2026-10-27 06:50 | Commit ab75ed1. Engine wired into il-server as the FlowRunner |
| 3 | B02 | Model gateway (Token Factory + fake) | P0 | T00 | done | claude-a | t/B02-gateway | 2026-10-27 12:25 | [PR #6](https://github.com/spark-456/interlock/pull/6). Live smoke test `cargo run -p il-ai --example ping` waits on NEBIUS_API_KEY |
| 4 | W01 | Design system & app shell | P0 | T00 | done | claude-a | t/W01-design-foundation | 2026-10-27 06:10 | Design v3 foundation merged (tokens, fonts, base components, shell, canvas foundation, Building). Remaining components moved to W01b. Commit 44aa61e |
| 5 | W02 | Canvas editor + Ask diff view | P0 | T00 | done | claude-a | t/W02-editor | 2026-10-27 13:55 | [PR #8](https://github.com/spark-456/interlock/pull/8). Editor at /flows/[id]; the node panel (W03) mounts beside the canvas |
| 6 | B01 | Store: SQLite, migrations, files, secrets | P0 | T00 | done | codex-1 | t/B01-store | 2026-10-27 13:48 | [PR #9](https://github.com/spark-456/interlock/pull/9) |
| 7 | A01 | Architect: goal → flow | P0 | T00 | done | claude-a | t/A01-architect | 2026-10-27 21:45 | [PR #11](https://github.com/spark-456/interlock/pull/11) |
| 8 | B06 | Nodes: logic & data | P0 | T00 | done | codex-1 | t/B06-logic-data | 2026-10-27 19:58 | [PR #15](https://github.com/spark-456/interlock/pull/15) squash merged after Ubuntu and Windows CI passed. |
| 9 | B07 | Nodes: AI | P0 | T00 | done | codex-1 | t/B07-ai | 2026-10-27 20:12 | [PR #16](https://github.com/spark-456/interlock/pull/16) squash merged after Ubuntu and Windows CI passed. |
| 10 | B04 | Server, API, SSE, auth, demo sessions | P0 | T00 | done | claude-a | t/B04-server | 2026-10-27 08:05 | Part 1, commit 9fcc8b3: all /api routes, SSE, demo sessions, files, CORS. Rest is in B04b |
| 11 | W03 | Node panel, schema forms, previews | P0 | T00 | done | claude-a | t/W03-panel | 2026-10-27 20:05 | [PR #10](https://github.com/spark-456/interlock/pull/10) |
| 12 | B05 | Triggers & scheduler | P0 | T00 | done | claude-a | t/B05-triggers | 2026-10-27 07:30 | Commit 0b96b2b. App triggers use `il_sdk::poll` |
| 13 | W04 | Create & Building screens | P0 | T00 | done | claude-a | t/W04-create | 2026-10-27 23:30 | [PR #13](https://github.com/spark-456/interlock/pull/13) |
| 14 | K01 | Knowledge: ingest, Tavily, hybrid search | P0 | T00 | done | codex-1 | t/K01-knowledge | 2026-10-27 21:20 | [PR #25](https://github.com/spark-456/interlock/pull/25) squash merged after local `cargo xtask check` and Ubuntu/Windows CI passed. |
| 15 | B08 | Nodes: human & output | P0 | T00 | done | codex-1 | t/B08-human-output | 2026-10-27 21:09 | [PR #24](https://github.com/spark-456/interlock/pull/24) squash merged after local `cargo xtask check` and Ubuntu/Windows CI passed. |
| 16 | P01 | Publish: OpenAI API, MCP, widget, guide | P0 | T00 | done | codex-1 | t/P01-publish | 2026-10-27 21:52 | [PR #29](https://github.com/spark-456/interlock/pull/29) squash merged after local `cargo xtask check` and Ubuntu/Windows CI passed. Official OpenAI JS SDK local-server test remains assigned to R03 in the web handoff. |
| 17 | R01 | Templates, fixtures, recorded runs, demo seed | P0 | T00 | done | claude-a | t/R01-samples | 2026-10-27 22:13 | [PR #30](https://github.com/spark-456/interlock/pull/30), [PR #32](https://github.com/spark-456/interlock/pull/32), [PR #33](https://github.com/spark-456/interlock/pull/33) |
| 18 | A02 | Architect: test-and-fix, Ask diff, app kit | P0 | A01 | done | claude-a | t/A02-testfix-ask | 2026-10-27 20:13 | [PR #17](https://github.com/spark-456/interlock/pull/17) |
| 19 | W05 | Runs, approvals, flows list, templates, connections | P0 | T00 | done | claude-a | t/W05-runs-2 | 2026-10-27 20:43 | [PR #18](https://github.com/spark-456/interlock/pull/18), [PR #20](https://github.com/spark-456/interlock/pull/20) |
| 20 | W06 | Publish/Connect & chat playground | P0 | T00 | done | claude-a | t/W06-chat | 2026-10-27 21:09 | [PR #22](https://github.com/spark-456/interlock/pull/22), [PR #23](https://github.com/spark-456/interlock/pull/23) |
| 21 | K02 | Use any API (OpenAPI → tools) | P0 | T00 | done | codex-1 | t/K02-openapi | 2026-10-27 21:36 | [PR #27](https://github.com/spark-456/interlock/pull/27) squash merged after local `cargo xtask check` and Ubuntu/Windows CI passed; Ubuntu's isolated Playwright drag failure passed on rerun. |
| 22 | C01 | OAuth broker + Gmail, Sheets, YouTube | P0 | T00 | done | codex-1 | t/C01-google | 2026-10-27 22:05 | [PR #31](https://github.com/spark-456/interlock/pull/31) squash merged after local `cargo xtask check` and Ubuntu/Windows CI passed. |
| 23 | C02 | Slack, Discord, Telegram, Bluesky | P0 | T00 | done | codex-1 | t/C02-messaging | 2026-10-27 22:17 | [PR #34](https://github.com/spark-456/interlock/pull/34) squash merged after local `cargo xtask check` and Ubuntu/Windows CI passed. |
| 24 | M01 | Media ops (ffmpeg) | P0 | T00 | done | codex-1 | t/M01-media-ops | 2026-10-27 22:30 | [PR #35](https://github.com/spark-456/interlock/pull/35) squash merged after local `cargo xtask check` and Ubuntu/Windows CI passed. |
| 25 | M02 | Media generation (video, voice, image) | P0 | T00 | done | codex-1 | t/M02-media-gen | 2026-10-27 23:06 | [PR #38](https://github.com/spark-456/interlock/pull/38); full `cargo xtask check` and Ubuntu/Windows CI passed; squash-merged as `13958172006cb5096c238b1e2a82eee99958997b`. |
| 26 | C03 | Shopify sandbox + social dry-run nodes | P0 | T00 | done | codex-1 | t/C03-sandbox-social | 2026-10-27 22:46 | [PR #36](https://github.com/spark-456/interlock/pull/36); full `cargo xtask check` and Ubuntu/Windows CI passed; squash-merged as `afaac62c788c6dd511d0df93df753282b5542a8c`. |
| 27 | W07 | Landing, demo mode, settings, error states | P0 | T00 | done | claude-a | t/W07-states | 2026-10-27 21:46 | [PR #26](https://github.com/spark-456/interlock/pull/26), [PR #28](https://github.com/spark-456/interlock/pull/28) |
| 28 | R02 | Deploy to Nebius | P0 | B04 | claimed | codex-1 | t/R02-deploy-nebius | 2026-10-27 23:45 | [Draft PR #39](https://github.com/spark-456/interlock/pull/39), branch `0d6128391dc6c6cb8fb942c1f23ebdd65a6562b0`: image, HTTPS Compose, runbook, smoke script done; local `cargo xtask check` and fake-provider smoke passed. Added the F-C sample OpenAPI spec to the image. Left: Docker image build (daemon unavailable) and public Nebius deploy with persistent disk, DNS, keys, external smoke; waiting on owner account/budget. |
| 29 | R03 | Integration pass & E2E | P0 | B03, B04, A01, W02, W03 | claimed | claude-a | t/R03-integration | 2026-10-27 23:08 | Part 1 merged ([PR #37](https://github.com/spark-456/interlock/pull/37)): fake-mode integration fixes, journey e2e; bugs filed as R05, R06. Left: run F-A/F-B/F-C on a staging deploy (needs R02 and the owner's keys) |
| 30 | R04 | README, docs, video script, Devpost | P0 | — | claimed | claude-a | t/R04-drafts | 2026-10-27 23:35 | Drafts merged ([PR #43](https://github.com/spark-456/interlock/pull/43)): docs/submission/video-script.md and devpost.md. Left for the owner at submission: README, Apache-2.0 switch, demo URL, video, real feedback |
| 31 | W01b | Remaining design components + `/dev/components` gallery | P0 | T00 | done | claude-a | t/W01b-components | 2026-10-27 23:31 | [PR #42](https://github.com/spark-456/interlock/pull/42) |
| 32 | N01 | Node: `ai.guardrails` (AR-9 safety rules) | P0 | T00 | done | codex-1 | t/N01-guardrails | 2026-10-27 20:42 | [PR #19](https://github.com/spark-456/interlock/pull/19) squash merged after Ubuntu and Windows CI passed. |
| 33 | B04b | Server: GitHub sign-in, rate limits, startup recovery, demo seed | P0 | T00 | done | codex-1 | t/B04b-server-finish | 2026-10-27 17:34 | [PR #12](https://github.com/spark-456/interlock/pull/12); Ubuntu and Windows CI passed |
| 34 | B03b | Engine: approval expiry and timeout policy | P0 | B03 | done | codex-1 | t/B03b-approval-expiry | 2026-10-27 20:56 | [PR #21](https://github.com/spark-456/interlock/pull/21) squash merged after Ubuntu and Windows CI passed; B08 node wiring is next. |
| 35 | T01 | Build size: lighter dev/test debug info | P1 | — | done | claude-a | t/T01-build-size | 2026-10-27 19:33 | [PR #14](https://github.com/spark-456/interlock/pull/14). Clean build target 3.0 GB (was 7–9 GB) |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | done | claude-a | t/C04-key-connections | 2026-10-27 23:58 | [PR #44](https://github.com/spark-456/interlock/pull/44) |
| 37 | U01 | Usage: spend by day and model role, editable caps (WS-5) | P1 | B04 | done | claude-a | t/U01-usage | 2026-10-28 00:14 | [PR #45](https://github.com/spark-456/interlock/pull/45): 30-day spend by day and flow, caps shown. Per-role spend and editable caps moved to U02 |
| 38 | R05 | Agents use the flow's OpenAPI tools (F-C invoices) | P0 | K02, P01 | review | codex-1 | t/R05-agent-openapi-tools | 2026-10-29 04:15 | [PR #55](https://github.com/spark-456/interlock/pull/55): agent `apiTools`, offline spec seeding, F-C invoice tools, risky-operation exclusion, generated catalog, and full local `IL_PROVIDERS=fake cargo xtask check` passed. Awaiting CI gate. |
| 39 | R06 | Builds in fake mode | P1 | A01 | done | codex-1 | t/R06-fake-builds | 2026-10-27 23:32 | [PR #41](https://github.com/spark-456/interlock/pull/41) squash merged after Ubuntu and Windows CI passed; local `cargo xtask check` passed. |
| 40 | U02 | Spend by model role and editable workspace caps | P2 | U01 | ready | | | | The gateway records each model call's role, workspace and cost in `us_ledger`; the engine reads per-workspace cap overrides (stored per workspace) before config; Settings lets owners edit them |
| 41 | R07 | Owner feedback: panel gaps, templates Off hint, app logos, UI/UX pass | P0 | R03 | done | claude-a | t/R07-panel-fixes | 2026-10-28 04:35 | Done in #48 (webhook address and trigger info, Save & test run, card warnings for missing connections, Off strip, quiet fake-mode log), #49 (Simple Icons logos), #50 (Test run on Off flow cards, Off hint, Connect only for Google, Slack bot token). LinkedIn/Cursor/VS Code keep letter tiles (not in Simple Icons). |
| 42 | R08 | Test runs never send or post | P0 | B03 | done | claude-a | t/R08-test-runs-safe | 2026-10-28 04:15 | PR #47 merged |
| 43 | Q01 | Real-model pass: verify model ids, build F-A/F-B/F-C with Nemotron, tune prompts | P0 | owner keys | ready |  |  |  | Needs NEBIUS_API_KEY. Fix `[models.roles]` ids from `GET /v1/models`; build the three hero flows from their sentences; tune `prompts/architect/*.md` and node prompts until builds validate first time; record real replies as golden fixtures |
| 44 | Q02 | Every template test-runs end to end with sample data | P0 | R01 | done | claude-a | t/Q02-template-samples | 2026-10-28 04:53 | PR #51 merged; CI fix in #52 |
| 45 | Q03 | Security review before the public deploy | P0 | R02 | ready |  |  |  | SSRF guard on every outbound call (incl. OpenAPI and HTTP steps), no secrets in logs or API responses, CORS and cookie flags on HTTPS, rate limits on /hooks, /v1, /mcp, /w and demo session, demo isolation; fix or file findings |
| 46 | Q04 | Demo workspaces reset daily | P1 | B04b | ready |  |  |  | The demo promises a daily reset: a background job deletes demo workspaces (flows, runs, files, credentials, knowledge) older than 24 h; test with a fake clock |
| 47 | Q05 | Agents tolerate missing tools | P1 | R05 | ready |  |  |  | An `ai.agent` naming a tool the flow doesn't provide fails the whole step (`tool get_invoice is unavailable`); skip unknown tools with a warning in the step's activity and tell the architect's validator to flag them |
| 48 | Q06 | Accessibility and mobile pass | P1 | W07 | ready |  |  |  | axe-core checks in Playwright on every screen (fail on serious issues); keyboard-only walk through create → build → editor → publish; 390 px layouts for the editor panel and playground per design M2–M4 |
| 49 | Q07 | Friendly run errors | P2 | B03 | ready |  |  |  | Map connector and engine errors to plain-language messages with a next step ("Connect Google in Connections", "This step needs a video from an earlier step") in the run view and node panel |
| 50 | Q08 | Parallel branches pair items when they join | P1 | B03 | ready |  |  |  | When two per-item branches from one step meet at a per-item step, the join sees 2×N separate items (one per branch) instead of N items carrying both branches' fields and files; and `nodes.<id>.json` is always the upstream step's first item, so per-item flows that read it get item 1's values. Pair items by their source item (zip on lineage) and expose the paired upstream item in expressions; add engine tests. Found in Q02 (promos had to be chained instead). |
| 51 | Q09 | Edit a trigger's sample data in the panel | P2 | Q02 | done | claude-a | t/Q09-trigger-samples | 2026-10-28 05:13 | PR #53 merged |
| 52 | B09 | Model fallbacks: Groq and OpenRouter | P1 | B02 | claimed | claude-a | t/B09-model-fallbacks | 2026-10-29 04:00 | Owner request: when Token Factory is rate-limited, down, out of credit or has no key, the gateway retries the same role on Groq, then OpenRouter (each only when its key is set) |
| 53 | C05 | Connectors: Notion, Airtable, HubSpot | P1 | C04 | claimed | claude-a | t/C05-key-connectors | 2026-10-29 04:00 | API-key apps from PRD P1: Notion create page / query database, Airtable create / list records, HubSpot create or update contact |
| 54 | C06 | Google Drive upload and new file; Sheets read rows | P1 | C01 | ready |  |  |  | Uses the Google sign-in from C01 |

## Waiting on the owner

Things only the owner can do (accounts, keys, money, decisions). Agents add a row when they need one and set Status `done` when it lands. The progress tracker reads this table.

| Item | Needed by | Detail | Status |
|---|---|---|---|
| Nebius Token Factory API key | B02, A01, R03 | Set `NEBIUS_API_KEY`. Check the model IDs in `config/interlock.example.toml` against `GET /v1/models` | open |
| Tavily API key | K01 | Set `TAVILY_API_KEY` | open |
| GitHub OAuth app | B04b | Client ID and secret; local callback `http://127.0.0.1:8080/api/auth/github/callback` | open |
| Google OAuth app (testing mode) | C01 | Gmail, Sheets and YouTube scopes; add yourself as a test user | open |
| Slack app | C02 | Bot token and an interactivity URL | open |
| Discord webhook | C02 | One webhook URL for a test channel | open |
| Telegram bot token | C02 | Create a bot with BotFather | open |
| Bluesky app password | C02 | Settings → App passwords | open |
| `IL_MASTER_KEY` | B01 | 32 random bytes, base64. Encrypts stored credentials | open |
| Nebius deployment account | R02 | Provide project access and VM/persistent-disk budget, public DNS name, and runtime keys via environment; then verify HTTPS demo access | open |
| Final licence and README | R04 | Switch to Apache-2.0 and write the README at submission | open |

## Blockers and known issues

| Issue | Kind | Tasks | Detail | Status |
|---|---|---|---|---|
| Generated node catalog fixture | handoff to claude-a | B06, B07, B08, N01, K01, K02, C01, C02, M01 and later node tasks | Owner chose claude-a to update `web/fixtures/api/nodes.json`; codex-1 leaves `web/` untouched. Claude: check out each pushed backend task branch, run `cargo xtask catalog`, commit only the generated web fixture on that branch, and note the SHA in its task row. Codex-1 will then run full check and open its PR. | done for current branches (claude-a, 2026-10-28). After each merge, write "needs catalog refresh" in the next row and claude-a regenerates it |
| Publish web handoff | handoff to claude-a | P01 | On `t/P01-publish`, run `cargo xtask gen-types` and commit generated `web/src/lib/api/types/{ApiKeySummary,CreateApiKeyRequest,CreateApiKeyResponse,index}.ts`; add the vanilla TS widget under `web/widget/` with streamed answers/sources and a local-server test using the official OpenAI JS SDK. codex-1 cannot edit `web/`; note the SHA in P01's task row. | done (`e0d3f6d`). The OpenAI JS SDK test moves to R03, which runs the real server |
| Approval generated types | handoff to claude-a | B03b | On `t/B03b-approval-expiry`, run `cargo xtask gen-types` and commit only `web/src/lib/api/types/{Approval,ApprovalTimeout,index}.ts`; `contracts/approval.json` is already in codex-1's branch. Note the SHA in B03b's row. | done (`5124a26`) |
| GitHub Actions blocked on private repos | blocker | all | Account billing stops hosted runners. The repo is public so CI runs; making it private again stops CI | workaround |
| Model IDs not verified | issue | B02 | Config model IDs are guesses until checked against Token Factory | open |
| Architect must emit `build.node` and `build.cost` | issue | A01, A02 | The Building screen needs them for node stats and the running cost | open |
| No `ai.guardrails` node | issue | N01 | The F-C flow and the design's "Safety rules" node need it | open |
| `corepack enable` needs admin | note | — | pnpm 10.14 works through the existing shims | open |

### Contract log (breaking changes only)

| Date (UTC) | PR | Change | Who must rebase |
|---|---|---|---|
| | | | |

---

## Task cards

Each card lists **Owns paths** (only edit inside these, plus tiny additive changes per AGENTS.md §4), **Soft deps** (use fakes until merged) and **Acceptance**. PRD references are in brackets.

### T00 · Contracts & skeleton
- **Goal**: everything every other task plugs into, compiling and green, so all other tasks can run in parallel.
- **Owns**: whole repo (initial).
- **Acceptance**:
  - Cargo workspace with all crates from PRD §7.2, each compiling, with the module folders every later task will fill (e.g. `il-nodes/src/{trigger,ai,knowledge,logic,data,human,output}/`, `il-connectors/src/{oauth,gmail,sheets,youtube,slack,discord,telegram,bluesky,shopify,social_dry_run,openapi}/`, `il-media/src/{ops,gen}/`, `il-architect/src/{compile,layout,catalog,validate,testfix,ask,appkit}/`, `il-engine/src/triggers/`) so later tasks never edit `lib.rs` module lists.
  - `il-core`: Flow, Node, Edge, Group, Item, FileRef, NodeManifest types, events (PRD §8.4), API DTOs, error shape, expression evaluation (minijinja) with tests.
  - `il-sdk`: `NodeExecutor`, `TriggerNode`, `NodeCtx`, service traits (`Store`, `ModelGateway`, `FileStore`, `KnowledgeService`, `Notifier`, `MediaGen`, `Clock`, `FlowRunner`, `ArchitectService`, `AgentTool`), `inventory` registries (nodes, triggers, providers, media gens, route factories), SSRF-safe HTTP client, `testkit` (in-memory store, fake gateway with scripted responses, fake files, fake HTTP, `NodeHarness`).
  - **Stub manifests for every P0 node** in PRD §5.5, each in the folder given by the PRD §5.5 placement table, registered, with real settings schemas and `NotImplemented` executors; `GET /api/nodes` returns the full catalog.
  - `il-server` boots, serves `/healthz`, `/api/nodes`, and the embedded web build; config loading from `config/interlock.toml` + env.
  - `xtask`: `dev`, `check`, `check-deps`, `gen-types`, `catalog`.
  - Web: SvelteKit SPA + TS strict + `@xyflow/svelte` installed; `src/lib/api` client + SSE helper; generated types; **fixture mode** (`VITE_FIXTURES=1`) serving `web/fixtures/` including one recorded build stream and one recorded run stream for F-A; placeholder routes for every screen in PRD §7.3; `tokens.css` with the design tokens.
  - CI (GitHub Actions) on `windows-latest` and `ubuntu-latest` running `cargo xtask check`.
  - `AGENTS.md`, `CLAUDE.md`, `docs/PRD.md`, `docs/WORKBOARD.md`, `docs/FUTURE.md`, `LICENSE` (Apache-2.0), README skeleton, `config/interlock.example.toml`, Dockerfile skeleton, `.gitignore`.

### B01 · Store
- **Owns**: `crates/il-store/**`, all `migrations/*.sql` except `*_kb_*` (owned by K01).
- **Soft deps**: none.
- **Acceptance**: rusqlite (bundled) async wrapper; WAL mode; migrations runner; repositories implementing `Store` for workspaces, users, API keys (hashed), flows + versions, runs, events (append + cursor read), node runs, approvals, schedules, webhooks, poll cursors, credentials (AES-256-GCM), builds, usage ledger; `FileStore` on disk with preview URLs; sqlite-vec loads (smoke test) and FTS5 is available; tests for each repository.

### B02 · Model gateway
- **Owns**: `crates/il-ai/**`.
- **Soft deps**: none (fake provider already in testkit).
- **Acceptance**: Token Factory provider (OpenAI-compatible: chat, streaming, tools, JSON-schema output, embeddings); roles → model IDs from config; startup check against `GET /v1/models` with clear logs; transparent fallback to JSON-in-prompt + validate/repair when tools/schema unsupported; retries with backoff; per-call token and cost metering to the current run/build; transcription via a configurable provider trait; wiremock tests; a `cargo run -p il-ai --example ping` live smoke test (needs `NEBIUS_API_KEY`).

### B03 · Engine
- **Owns**: `crates/il-engine/**` except `src/triggers/`.
- **Soft deps**: B01 (use testkit in-memory store).
- **Acceptance** [EN-1…EN-4, EN-6…EN-8]: executor over the flow graph (per-item/batch, named branch outputs, merges, parallel branches with concurrency limit); expression rendering per item; retries, timeouts, cancellation; Human-node pause → persisted → resume on decision (survives restart); spend caps; ordered event emission to the store and a broadcast bus; run replay from stored events; tests covering all of these with fake nodes.

### B04 · Server, API, SSE, auth, demo
- **Owns**: `crates/il-server/**`.
- **Soft deps**: B01, B03 (use testkit implementations until merged).
- **Acceptance** [PRD §8.5, WS-1, WS-2]: all internal `/api` routes wired to traits; SSE for runs and builds with `after` cursor and heartbeat; auto-mount of registered route factories (publish, oauth, hooks); GitHub OAuth sign-in with cookie sessions; demo sessions (ephemeral workspace, caps, seeded templates); multipart upload; consistent error shape; CORS; compression; rate limiting on public endpoints; integration tests with an in-memory setup.

### B05 · Triggers & scheduler
- **Owns**: `crates/il-engine/src/triggers/**`, `crates/il-nodes/src/trigger/**`.
- **Soft deps**: B03, B04.
- **Acceptance** [EN-5]: trigger supervisor that starts/stops triggers when flows activate/deactivate; Manual, Schedule (cron + timezone), Webhook (token URL → run), polling framework (interval + cursor) used by app triggers, Chat/API/MCP invocation entry point returning the Respond node's output; tests with a fake clock.

### B06 · Nodes: logic & data
- **Owns**: `crates/il-nodes/src/logic/**`, `crates/il-nodes/src/data/**`.
- **Soft deps**: none.
- **Acceptance**: If, Switch (dynamic outputs), Loop over items (split a list field into items), Merge (append/by key/wait all), Wait (duration/until), Stop; Set fields, Template, HTTP request (SSRF-safe, auth options, JSON/text/binary), Parse document (delegates to the knowledge parsers trait, fake until K01); each with manifest examples and harness tests.

### B07 · Nodes: AI
- **Owns**: `crates/il-nodes/src/ai/**`, `prompts/nodes/**`.
- **Soft deps**: B02 (fake gateway).
- **Acceptance**: Write, Summarise, Classify (labels → named outputs), Extract fields (JSON schema output), Translate, Decide (yes/no or labels), Agent (tool loop: knowledge search tool, node-provided tools, OpenAPI tools via trait; max steps; returns answer + sources + activity), Describe image, Transcribe audio; all use roles (no model IDs); prompts in `prompts/nodes/`; snapshot tests of prompts; harness tests with scripted fake responses.

### B08 · Nodes: human & output
- **Owns**: `crates/il-nodes/src/human/**`, `crates/il-nodes/src/output/**`.
- **Soft deps**: B03.
- **Acceptance**: Approval (in-app; channel adapters via `Notifier` trait — Slack adapter lives in C02), with Approved/Rejected outputs, editable payload, timeout behaviour; Ask a question; Respond (for chat/API/MCP flows, streaming-capable); Send notification; harness tests.

### B03b · Engine: approval expiry and timeout policy
- **Owns**: `crates/il-engine/src/approvals/**`, `crates/il-engine/src/executor/**`; small additive approval contract and store changes as needed.
- **Hard deps**: B03.
- **Acceptance**: persist the approval deadline and `onTimeout` policy; after the deadline, approve, reject or keep waiting as configured; resume the correct run and item after a restart; reject late manual decisions; tests use a fake clock and store with no network.

### K01 · Knowledge
- **Owns**: `crates/il-knowledge/**`, `crates/il-nodes/src/knowledge/**`, `migrations/*_kb_*.sql`.
- **Soft deps**: B01, B02.
- **Acceptance** [KN-1…KN-5]: parsers (PDF text layer, DOCX, MD, TXT, CSV, HTML); Tavily crawl/extract client (cap 50 pages, respects limits); transcription → Knowledge or Instructions; chunking with title/section/date metadata; embeddings via gateway; sqlite-vec + FTS5 hybrid search with reciprocal-rank fusion; `KnowledgeService` impl; nodes Knowledge base, Search knowledge, Add to knowledge; tests with fixtures (no network).

### K02 · Use any API (OpenAPI → tools)
- **Owns**: `crates/il-connectors/src/openapi/**`.
- **Soft deps**: none.
- **Acceptance**: parse OpenAPI 3.0/3.1 (URL or file) and cURL examples; list operations with risk flags (DELETE/destructive → skipped unless allowed); endpoint selection helper for the architect; "Use any API" node (pick operation, map params, auth via credential); tool adapter so the Agent node can call selected operations; tests with sample specs in `fixtures/apis/`.

### P01 · Publish
- **Owns**: `crates/il-publish/**`, `web/widget/**`.
- **Soft deps**: B03, B04.
- **Acceptance** [PB-1…PB-6]: `POST /v1/chat/completions` (stream + non-stream) and `GET /v1/models` backed by published AI-app flows; per-flow MCP (rmcp, streamable HTTP) with `ask_<slug>`, search and tool exposure; platform MCP (`list_flows`, `get_integration_guide`, `add_documents`, `create_flow_from_goal`, `run_flow`); widget bundle (vanilla TS, tiny, streams with sources); `INTERLOCK.md` generator with cURL/JS/Python/Rust; API keys (create/reveal once/revoke); tests including the official OpenAI JS SDK against a local server.

### C01 · OAuth broker + Google
- **Owns**: `crates/il-connectors/src/{oauth,gmail,sheets,youtube}/**`.
- **Soft deps**: B01, B04, B05.
- **Acceptance**: generic OAuth2 broker (start/callback/refresh, encrypted tokens, provider configs self-registered); Gmail: new-email trigger (polling with cursor), reply, send, label; Sheets: append row; YouTube: upload (private); all with dedupe keys and harness tests using wiremock.

### C02 · Messaging
- **Owns**: `crates/il-connectors/src/{slack,discord,telegram,bluesky}/**`.
- **Soft deps**: B08.
- **Acceptance**: Slack (OAuth or bot token): send message, approval with interactive buttons → decision callback route; Discord: webhook/bot send with attachments; Telegram: bot send with media; Bluesky: post with video/image via app password; `Notifier` adapters; tests with wiremock.

### C03 · Shopify sandbox + social dry-run
- **Owns**: `crates/il-connectors/src/{shopify,social_dry_run}/**`, `fixtures/sandbox-store/**`.
- **Soft deps**: none.
- **Acceptance**: built-in sandbox store (fictional products with images) behind the same node interface as real Shopify (real Shopify via Admin API token optional); "New product" trigger and "Get products"; Instagram, TikTok, LinkedIn, X nodes with full manifests, input validation, a rendered post preview in the output, `status: DryRun`, and a clear note "Needs platform approval to post"; tests.

### M01 · Media ops
- **Owns**: `crates/il-media/src/ops/**`.
- **Soft deps**: none.
- **Acceptance**: ffmpeg wrappers (child process, progress parsing, timeouts): cut clips, resize for platform (9:16, 1:1, 16:9 with smart crop/pad), add captions (burned in from text/SRT), merge audio and video, thumbnail, convert; nodes registered; tests with tiny fixture media (skipped if ffmpeg is missing, with a clear message).

### M02 · Media generation
- **Owns**: `crates/il-media/src/gen/**`.
- **Soft deps**: none.
- **Acceptance**: `MediaGen` adapters for video, voice and image, each written to its provider's documented API and switched on only when that provider's key is configured (one real adapter each, chosen in config). **Live-provider testing is out of scope for the hackathon**: the build only shows that real generation is possible. A `recorded` provider returning fixture media is the default in demo mode, in test runs and in tests. Nodes Generate video, Generate voice, Generate image with progress events and cost reporting; adapter tests against wiremock only.

### A01 · Architect: goal → flow
- **Owns**: `crates/il-architect/src/{compile,layout,catalog,validate}/**`, `prompts/architect/**`.
- **Soft deps**: B02 (scripted fake model).
- **Acceptance** [AR-1, AR-3…AR-5, AR-7, AR-10]: builds a compact catalog from registered manifests; compiles goal + attachments into Flow JSON with the Architect role; validates (schema, known types, settings schemas, edges/ports, expressions reference real nodes); repair loop with errors (max 3); clarifying questions when required values are missing (AR-2, P1); layered auto-layout with groups; connection check; emits build events incrementally (`build.flow_patch` per node/edge); golden tests for F-A, F-B, F-C goals with scripted responses; a live example run behind a flag.

### A02 · Architect: test-and-fix, Ask diff, app kit
- **Owns**: `crates/il-architect/src/{testfix,ask,appkit}/**`.
- **Hard deps**: A01. **Soft deps**: B03.
- **Acceptance** [AR-6, AR-8, AR-9]: test run with sample data through the engine trait, failure analysis, up to 2 fix rounds with `build.fix` events; Ask Interlock → JSON-Patch diff with summary ("+1 step, 1 changed"); for AI apps, generate guardrails and a test suite (knowledge questions + tool tasks), run it, and fix prompts on failure; tests.

### W01 · Design system & app shell
- **Owns**: `web/src/lib/design/**`, `web/src/routes/+layout.svelte`, `web/src/app.css`, `web/src/app.html`.
- **Soft deps**: Claude Design export in `design/`.
- **Acceptance**: tokens and every base component from the design (Button, IconButton, Input, TextArea, Select, SegmentedControl, Toggle, Checkbox, Chip, StatusPill, KindBadge, Tabs, Tooltip, Popover, Menu, Modal, BottomSheet, Toast, Banner, AppTile, PartnerSlot, BuiltWithStrip, EmptyState, Skeleton, CopyField, CodeTabs), icon set, sidebar + top bar + mobile tab bar, light theme (dark P1), a `/dev/components` gallery page; vitest for interactive components.

### W02 · Canvas editor + Ask diff view
- **Owns**: `web/src/lib/canvas/**`, `web/src/routes/flows/[id]/+page.svelte`.
- **Soft deps**: W01, W03 (fixture mode).
- **Acceptance** [ED-1, ED-2, ED-6, ED-9]: Svelte Flow canvas with NodeCard (all states from the design), edges with running animation and counts, groups, minimap, toolbar (zoom, fit, tidy via API, undo/redo), palette from `/api/nodes` with search and drag-to-add, insert on edge, Replace with… (keeps compatible settings/mappings), list view, Ask Interlock bar + diff rendering (added/changed/removed) with accept/discard; renders live state from the run SSE stream; works fully in fixture mode.

### W03 · Node panel, schema forms, previews
- **Owns**: `web/src/lib/panel/**`, `web/src/lib/forms/**`, `web/src/lib/previews/**`.
- **Soft deps**: W01.
- **Acceptance** [ED-3…ED-5, ED-7]: JSON-Schema form renderer (string, number, boolean, enum, array, object, oneOf) with `x-widget` hints (expression, credential, file, longtext, select, model-role); field mapper chips with an expression picker listing upstream outputs; credential picker with "Connect"; Input/Output tabs with preview renderers (text, email, image, video, audio, table, JSON) auto-discovered; "Run this step"; vitest coverage for the form renderer.

### W04 · Create & Building
- **Owns**: `web/src/routes/new/**`.
- **Soft deps**: W01, W02.
- **Acceptance** [AR-1, AR-7]: prompt box with example chips, attachments (files, links, audio) with Knowledge/Instructions toggles, voice input (browser speech recognition where available; otherwise record and upload for transcription), detected apps row, clarifying questions card; Building screen rendering the canvas incrementally from build SSE events, stepper, log, cost/time; "Open editor" on ready; works from the recorded build fixture.

### W05 · Runs, approvals, flows, templates, connections
- **Owns**: `web/src/routes/flows/+page.svelte`, `web/src/routes/flows/[id]/runs/**`, `web/src/routes/approvals/**`, `web/src/routes/templates/**`, `web/src/routes/connections/**`.
- **Soft deps**: W01, W02, W03.
- **Acceptance**: run view (canvas read-only + step list + clip gallery + approval card), approvals inbox (mobile-first), flows list with filters and sparklines, templates gallery with "Use template", connections grid with OAuth start and detail drawer; mobile layouts per design.

### W06 · Publish/Connect & chat
- **Owns**: `web/src/routes/flows/[id]/publish/**`, `web/src/routes/flows/[id]/chat/**`.
- **Soft deps**: W01.
- **Acceptance** [PB-*]: Activate state for automations; Publish page for AI apps with API, MCP and widget cards, reveal-once key, quick-start code tabs, integration guide preview and copy actions, platform MCP section; chat playground with streaming, activity chips, source chips with passage preview, suggested actions, "Try asking", and path highlight.

### W07 · Landing, demo, settings, states
- **Owns**: `web/src/routes/+page.svelte`, `web/src/routes/demo/**`, `web/src/routes/settings/**`, `web/src/routes/+error.svelte`.
- **Soft deps**: W01.
- **Acceptance** [MK-1, WS-2, WS-5]: landing page per design (desktop + mobile); demo entry creating a demo session and banner; settings (workspace, API keys, usage and caps); 404, error, offline banner, cap-reached states.

### R01 · Templates, fixtures, recorded runs, demo seed
- **Owns**: `fixtures/templates/**`, `fixtures/recorded/**`, `fixtures/docs/**`, `fixtures/apis/**`, `web/fixtures/**`.
- **Soft deps**: node stubs from T00 are enough to author templates.
- **Acceptance** [MK-2, MK-3]: 10 templates as Flow JSON validated against the catalog (F-A, F-B, F-C + 7 from the design list); sample documents, a short sample audio memo, the Larkspur OpenAPI spec and a tiny mock API; recorded event streams + media for F-A and F-C; demo seeding on first boot; a validation test that loads every template.

### R02 · Deploy to Nebius
- **Owns**: `deploy/**`, `Dockerfile`, `.github/workflows/deploy.yml`.
- **Hard deps**: B04.
- **Acceptance**: multi-stage image (web build → Rust release → slim runtime with ffmpeg, non-root, `/data` volume); deployed on Nebius with HTTPS and a persistent volume; secrets set; global spend cap on; public access for judges confirmed (no token wall); `deploy/README.md` runbook; smoke test script hitting `/healthz`, `/api/nodes`, demo session and a template run.

### R03 · Integration pass & E2E
- **Owns**: `web/e2e/**`, `tests/e2e/**`; cross-cutting fixes by agreement (note each in the PR).
- **Hard deps**: B03, B04, A01, W02, W03.
- **Acceptance**: F-A, F-B, F-C run end to end against real services on a staging deploy; fixture mode turned off in production builds; Playwright smoke (create → building → editor → test run → publish → chat) green in CI using fake providers; a bug list turned into new task rows.

### W01b · Remaining design components + gallery
- **Owns**: `web/src/lib/design/**`, `web/src/routes/dev/**`.
- **Acceptance**: the W01 components not yet built (Checkbox, KindBadge, Tooltip, Menu, EmptyState, Skeleton, CodeTabs, DropZone, FileRow with the Knowledge/Instructions toggle, ApiCard), matching `design/interlock/screens/SpecV3.png`; a `/dev/components` gallery showing every state; vitest for interactive components.

### N01 · Node: ai.guardrails
- **Owns**: `crates/il-nodes/src/ai/guardrails.rs` (+ its line in `ai/mod.rs`).
- **Acceptance**: manifest (rules array, hand-off rules such as "refunds over $100", outputs `pass`/`blocked`), harness tests with the fake gateway, and `cargo xtask catalog` regenerated. The architect uses it for F-C (AR-9). The design's "Safety rules" node maps to it (see `design/interlock/MAPPING.md`).

### B04b · Server: GitHub sign-in, rate limits, startup recovery, demo seed
- **Owns**: `crates/il-server/**`.
- **Acceptance**: GitHub OAuth sign-in (`/api/auth/github/start`, callback, creates a `Session` in the store like `POST /api/demo/session` does, cookie `il_session`); per-IP rate limiting on `/hooks`, `/v1`, `/mcp`, `/api/demo/session`; startup recovery (`engine.recover` for runs left Running, `triggers.restore` for active flows: add the store queries this needs with B01); demo sessions seeded with templates once R01 lands; integration tests.

### T01 · Build size: lighter dev/test debug info
- **Owns**: root `Cargo.toml` profiles.
- **Acceptance**: dev and test builds keep only line-table debug info for workspace crates and none for dependencies, so panics and backtraces still show file and line; `target` shrinks substantially on Windows; `cargo xtask check` and CI pass. CI already runs without incremental builds (`dtolnay/rust-toolchain` sets `CARGO_INCREMENTAL=0`).

### R04 · README, docs, video script, Devpost
- **Owns**: `README.md`, `docs/**` (except PRD and WORKBOARD), `assets/**`.
- **Acceptance** [PRD §13]: README with what/why, demo link, screenshots, architecture diagram, setup for Windows and Linux, how Nemotron Ultra/Super/Nano and Token Factory are used (with file paths), Tavily usage, Nebius deployment, honest limitations; `docs/NODES.md` generated; a demo video script under 3 minutes; Devpost text including the Tilo lineage note and feedback on Nebius/NVIDIA tools.
- **Licence (owner decision, 2026-10-27)**: the repo is public under an all-rights-reserved `LICENSE` for now. The submission checklist (PRD §13) asks for Apache-2.0 visible in the About section, so switch `LICENSE`, `Cargo.toml`, `web/package.json` and the README back to Apache-2.0 at submission (owner's decision). The root `README.md` was removed on 2026-10-27 so the public repo page stays bare (owner's decision); write it again from scratch for submission (its last version is in git history, commit 7c53926).

### B09 · Model fallbacks: Groq and OpenRouter
- **Owns**: `crates/il-ai/**`, `[models]` in `config/interlock.example.toml`.
- **Acceptance**: Groq and OpenRouter providers (OpenAI-compatible, keys `GROQ_API_KEY`, `OPENROUTER_API_KEY`) self-register; `[[models.fallbacks]]` lists providers with their own role → model ids; the gateway moves a call to the next provider after retries are used up on a rate limit, outage, missing or rejected key, or exhausted credit; providers without a key are skipped; Token Factory stays first (hackathon rule); wiremock tests for the fall-over.

### C05 · Connectors: Notion, Airtable, HubSpot
- **Owns**: `crates/il-connectors/src/{notion,airtable,hubspot}/`.
- **Acceptance**: `app.notion.create_page`, `app.notion.query_database`, `app.airtable.create_record`, `app.airtable.list_records`, `app.hubspot.upsert_contact`; API-key credentials saved from /connections (C04); destinations come only from settings; wiremock harness tests; `docs/NODES.md` regenerated.

### C06 · Google Drive upload and new file; Sheets read rows
- **Owns**: `crates/il-connectors/src/{drive,sheets}/`.
- **Acceptance**: `app.drive.upload` (file from an item into a folder), `trigger.drive.new_file` (polling with a cursor, like Gmail new email), `app.sheets.read_rows`; Google OAuth scopes added; wiremock harness tests; catalog regenerated.

---

## Suggested parallel plan (not binding — self-claiming still applies)

| When (IST) | claude-a (Windows, local) | claude-b (local worktree) | codex-1 | codex-2 |
|---|---|---|---|---|
| Tue morning | T00 | — (design export into `design/`) | — | — |
| Tue afternoon–night | B03 → B05 | W01 → W02 | B02 → B07 | B01 → B06 |
| Wed | B04 → B08 | W03 → W04 | A01 → A02 | K01 → K02 → R01 |
| Thu | R03 (integrator) + R02 prep | W05 → W06 → W07 | P01 → C02 | C01 → M01 → M02 → C03 |
| Fri | R03 → R02 deploy | polish + mobile | fixes from R03 | R04 |

Keep one agent on the critical path at all times: **T00 → B03 → B04 → R03 → R02**.

Good fits for cloud agents (well-specified, self-contained): B01, B02, B06, B07, K01, K02, C01–C03, M01, M02, P01, R01. Good fits for local agents (visual checks, Windows build, integration): T00, W01–W07, B04, R02, R03.
