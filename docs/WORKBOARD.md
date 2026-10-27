# WORKBOARD — Interlock
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
How to use this board: see `AGENTS.md` §3. Take the **first `ready` task from the top** whose hard deps are all `done`, claim it by editing its row, and push. **After T00 merges, every other task's only hard dependency is T00** (except A02, R02 and R03), so any free agent can take any task; soft deps are covered by fakes and fixtures.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
Status values: `ready` · `claimed` · `review` · `done` · `blocked`
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
Owner names: `claude-a`, `claude-b`, `codex-1`, `codex-2`, `human` (add more as needed)
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
## Index (priority order — top first)
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| # | ID | Task | Prio | Hard deps | Status | Owner | Branch | Updated (UTC) | PR / Notes |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
|---|---|---|---|---|---|---|---|---|---|
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 1 | T00 | Contracts & skeleton | P0 | — | done | claude-a | t/T00-skeleton | 2026-10-27 05:15 | Merged locally (no remote yet), commit 90af90f. Read its commit message for notes to later tasks |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 2 | B03 | Engine: executor, events, approvals, caps | P0 | T00 | done | claude-a | t/B03-engine | 2026-10-27 06:50 | Commit ab75ed1. Engine wired into il-server as the FlowRunner |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 3 | B02 | Model gateway (Token Factory + fake) | P0 | T00 | done | claude-a | t/B02-gateway | 2026-10-27 12:25 | [PR #6](https://github.com/spark-456/interlock/pull/6). Live smoke test `cargo run -p il-ai --example ping` waits on NEBIUS_API_KEY |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 4 | W01 | Design system & app shell | P0 | T00 | done | claude-a | t/W01-design-foundation | 2026-10-27 06:10 | Design v3 foundation merged (tokens, fonts, base components, shell, canvas foundation, Building). Remaining components moved to W01b. Commit 44aa61e |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 5 | W02 | Canvas editor + Ask diff view | P0 | T00 | done | claude-a | t/W02-editor | 2026-10-27 13:55 | [PR #8](https://github.com/spark-456/interlock/pull/8). Editor at /flows/[id]; the node panel (W03) mounts beside the canvas |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 6 | B01 | Store: SQLite, migrations, files, secrets | P0 | T00 | done | codex-1 | t/B01-store | 2026-10-27 13:48 | [PR #9](https://github.com/spark-456/interlock/pull/9) |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 7 | A01 | Architect: goal → flow | P0 | T00 | done | claude-a | t/A01-architect | 2026-10-27 21:45 | [PR #11](https://github.com/spark-456/interlock/pull/11) |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 8 | B06 | Nodes: logic & data | P0 | T00 | done | codex-1 | t/B06-logic-data | 2026-10-27 19:58 | [PR #15](https://github.com/spark-456/interlock/pull/15) squash merged after Ubuntu and Windows CI passed. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 9 | B07 | Nodes: AI | P0 | T00 | done | codex-1 | t/B07-ai | 2026-10-27 20:12 | [PR #16](https://github.com/spark-456/interlock/pull/16) squash merged after Ubuntu and Windows CI passed. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 10 | B04 | Server, API, SSE, auth, demo sessions | P0 | T00 | done | claude-a | t/B04-server | 2026-10-27 08:05 | Part 1, commit 9fcc8b3: all /api routes, SSE, demo sessions, files, CORS. Rest is in B04b |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 11 | W03 | Node panel, schema forms, previews | P0 | T00 | done | claude-a | t/W03-panel | 2026-10-27 20:05 | [PR #10](https://github.com/spark-456/interlock/pull/10) |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 12 | B05 | Triggers & scheduler | P0 | T00 | done | claude-a | t/B05-triggers | 2026-10-27 07:30 | Commit 0b96b2b. App triggers use `il_sdk::poll` |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 13 | W04 | Create & Building screens | P0 | T00 | done | claude-a | t/W04-create | 2026-10-27 23:30 | [PR #13](https://github.com/spark-456/interlock/pull/13) |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 14 | K01 | Knowledge: ingest, Tavily, hybrid search | P0 | T00 | claimed | codex-1 | t/K01-knowledge | 2026-10-27 18:18 | Pushed `0afe3b4d6ff1636a40ba9fe38f12bd4646625e33`; includes workspace ownership check needed by P01, nine knowledge tests and Clippy pass. Claude-a updates generated `web/fixtures/api/nodes.json` per handoff; then rerun full check and open PR. Catalog fixture `8e4683a` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 15 | B08 | Nodes: human & output | P0 | T00 | claimed | codex-1 | t/B08-human-output | 2026-10-27 20:17 | Rebased branch `98cf6c5` includes claude-a's fixture; full `cargo xtask check` passed. Remaining: `onTimeout` is in the manifest but not passed to the approval service; B03b must add expiry policy and resume behavior, then B08 needs the small node wiring and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 16 | P01 | Publish: OpenAI API, MCP, widget, guide | P0 | T00 | claimed | codex-1 | t/P01-publish | 2026-10-27 20:02 | Pushed `997f8fb` including claude-a's `e0d3f6d` widget/types handoff: public published-app-only widget chat, final-chunk sources, route rate limit and cross-origin preflight. Publish tests (8), focused server CORS test and Clippy pass. Left: production FlowToolProvider after K02, rebase K01 `owns_base`, official OpenAI SDK test, full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 17 | R01 | Templates, fixtures, recorded runs, demo seed | P0 | T00 | ready | | | | |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 18 | A02 | Architect: test-and-fix, Ask diff, app kit | P0 | A01 | done | claude-a | t/A02-testfix-ask | 2026-10-27 20:13 | [PR #17](https://github.com/spark-456/interlock/pull/17) |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 19 | W05 | Runs, approvals, flows list, templates, connections | P0 | T00 | claimed | claude-a | t/W05-runs | 2026-10-27 20:07 | Part 1 merged ([PR #18](https://github.com/spark-456/interlock/pull/18)); part 2 (run view, approvals, connections) on `t/W05-runs-2` |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 20 | W06 | Publish/Connect & chat playground | P0 | T00 | ready | | | | |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 21 | K02 | Use any API (OpenAPI → tools) | P0 | T00 | claimed | codex-1 | t/K02-openapi | 2026-10-27 19:28 | Pushed `c3547bd`: selected OpenAPI flow tools, scoped credentials, shared capped spec loader; 8 focused tests and Clippy pass. Needs catalog refresh by claude-a on this branch, then rebase main, full check and PR. Catalog fixture `9547cee` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 22 | C01 | OAuth broker + Gmail, Sheets, YouTube | P0 | T00 | claimed | codex-1 | t/C01-google | 2026-10-27 17:45 | Pushed `caeb4889fd97cade99896cf332c87e829d66d8d0`; OAuth routes, Google nodes, 27 connector tests and Clippy pass. Full check reaches generated-file drift at `web/fixtures/api/nodes.json`; claude-a updates it per handoff row, then rerun check and open PR. Catalog fixture `8a6bfc4` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 23 | C02 | Slack, Discord, Telegram, Bluesky | P0 | T00 | claimed | codex-1 | t/C02-messaging | 2026-10-27 16:12 | 77bdf615918bb4f10c9f61bb808c6e1d7aae47f9 pushed; 25 connector tests and Rust check stages pass; generated `web/fixtures/api/nodes.json` drift blocks full check under no-web instruction; PR pending. Catalog fixture `e938718` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 24 | M01 | Media ops (ffmpeg) | P0 | T00 | claimed | codex-1 | t/M01-media-ops | 2026-10-27 16:32 | 6295632eff30c793c55e1698010ebdbd5aa420ed pushed; Windows ffmpeg fixture and Rust check stages pass. Claude-a owns generated `web/fixtures/api/nodes.json` on this branch; full check and PR follow that commit. Catalog fixture `5c4e253` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 25 | M02 | Media generation (video, voice, image) | P0 | T00 | claimed | codex-1 | t/M02-media-gen | 2026-10-27 17:05 | Pushed `92085e45290dfc7029804127990d41178cea1f8d`: Runway and recorded adapters, nodes, wiremock tests. Rust check stages pass; generated `web/fixtures/api/nodes.json` drift awaits claude-a per handoff row. Then rerun full check and open PR. Catalog fixture `ba03ef4` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 26 | C03 | Shopify sandbox + social dry-run nodes | P0 | T00 | claimed | codex-1 | t/C03-sandbox-social | 2026-10-27 16:49 | 22f3e9ea0fa6c04f03af3542421038282eab5086 pushed; five C03 tests and Rust check stages pass. Claude-a owns generated `web/fixtures/api/nodes.json` on this branch; full check and PR follow that commit. Catalog fixture `f7a6316` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 27 | W07 | Landing, demo mode, settings, error states | P0 | T00 | ready | | | | |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 28 | R02 | Deploy to Nebius | P0 | B04 | ready | | | | Prep (Dockerfile, account checks) can start before B04 is done |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 29 | R03 | Integration pass & E2E | P0 | B03, B04, A01, W02, W03 | ready | | | | Integrator role; start Thu |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 30 | R04 | README, docs, video script, Devpost | P0 | — | ready | | | | Draft any time; finalise Fri |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 31 | W01b | Remaining design components + `/dev/components` gallery | P0 | T00 | ready | | | | Checkbox, KindBadge, Tooltip, Menu, EmptyState, Skeleton, CodeTabs, DropZone, FileRow, ApiCard; gallery page with every SpecV3 state; vitest for interactive components |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 32 | N01 | Node: `ai.guardrails` (AR-9 safety rules) | P0 | T00 | claimed | codex-1 | t/N01-guardrails | 2026-10-27 14:38 | Node, three fake-gateway tests, and generated docs pushed at 16d9560; full check and PR need generated `web/` fixture decision Catalog fixture `b86424c` pushed by claude-a; ready for full check and PR. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 33 | B04b | Server: GitHub sign-in, rate limits, startup recovery, demo seed | P0 | T00 | done | codex-1 | t/B04b-server-finish | 2026-10-27 17:34 | [PR #12](https://github.com/spark-456/interlock/pull/12); Ubuntu and Windows CI passed |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 34 | B03b | Engine: approval expiry and timeout policy | P0 | B03 | claimed | codex-1 | t/B03b-approval-expiry | 2026-10-27 20:26 | Pushed `645ba53`: persisted deadline/policy, atomic decisions, five-second expiry scan, restart and fake-clock tests. Rust stages of `cargo xtask check` pass; needs catalog refresh (generated TS types only). `contracts/approval.json` is committed. After claude-a's type commit: full check, PR, merge; then finish B08 node wiring. |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| 35 | T01 | Build size: lighter dev/test debug info | P1 | — | done | claude-a | t/T01-build-size | 2026-10-27 19:33 | [PR #14](https://github.com/spark-456/interlock/pull/14). Clean build target 3.0 GB (was 7–9 GB) |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
## Waiting on the owner
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
Things only the owner can do (accounts, keys, money, decisions). Agents add a row when they need one and set Status `done` when it lands. The progress tracker reads this table.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Item | Needed by | Detail | Status |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
|---|---|---|---|
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Nebius Token Factory API key | B02, A01, R03 | Set `NEBIUS_API_KEY`. Check the model IDs in `config/interlock.example.toml` against `GET /v1/models` | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Tavily API key | K01 | Set `TAVILY_API_KEY` | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| GitHub OAuth app | B04b | Client ID and secret; local callback `http://127.0.0.1:8080/api/auth/github/callback` | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Google OAuth app (testing mode) | C01 | Gmail, Sheets and YouTube scopes; add yourself as a test user | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Slack app | C02 | Bot token and an interactivity URL | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Discord webhook | C02 | One webhook URL for a test channel | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Telegram bot token | C02 | Create a bot with BotFather | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Bluesky app password | C02 | Settings → App passwords | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| `IL_MASTER_KEY` | B01 | 32 random bytes, base64. Encrypts stored credentials | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Nebius deployment account | R02 | Somewhere to run the container with HTTPS and a volume | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Final licence and README | R04 | Switch to Apache-2.0 and write the README at submission | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
## Blockers and known issues
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Issue | Kind | Tasks | Detail | Status |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
|---|---|---|---|---|
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Generated node catalog fixture | handoff to claude-a | B06, B07, B08, N01, K01, K02, C01, C02, M01 and later node tasks | Owner chose claude-a to update `web/fixtures/api/nodes.json`; codex-1 leaves `web/` untouched. Claude: check out each pushed backend task branch, run `cargo xtask catalog`, commit only the generated web fixture on that branch, and note the SHA in its task row. Codex-1 will then run full check and open its PR. | done for current branches (claude-a, 2026-10-28). After each merge, write "needs catalog refresh" in the next row and claude-a regenerates it |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Publish web handoff | handoff to claude-a | P01 | On `t/P01-publish`, run `cargo xtask gen-types` and commit generated `web/src/lib/api/types/{ApiKeySummary,CreateApiKeyRequest,CreateApiKeyResponse,index}.ts`; add the vanilla TS widget under `web/widget/` with streamed answers/sources and a local-server test using the official OpenAI JS SDK. codex-1 cannot edit `web/`; note the SHA in P01's task row. | done (`e0d3f6d`). The OpenAI JS SDK test moves to R03, which runs the real server |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Approval generated types | handoff to claude-a | B03b | On `t/B03b-approval-expiry`, run `cargo xtask gen-types` and commit only `web/src/lib/api/types/{Approval,ApprovalTimeout,index}.ts`; `contracts/approval.json` is already in codex-1's branch. Note the SHA in B03b's row. | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| GitHub Actions blocked on private repos | blocker | all | Account billing stops hosted runners. The repo is public so CI runs; making it private again stops CI | workaround |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Model IDs not verified | issue | B02 | Config model IDs are guesses until checked against Token Factory | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Architect must emit `build.node` and `build.cost` | issue | A01, A02 | The Building screen needs them for node stats and the running cost | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| No `ai.guardrails` node | issue | N01 | The F-C flow and the design's "Safety rules" node need it | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| `corepack enable` needs admin | note | — | pnpm 10.14 works through the existing shims | open |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### Contract log (breaking changes only)
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Date (UTC) | PR | Change | Who must rebase |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
|---|---|---|---|
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| | | | |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
---
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
## Task cards
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
Each card lists **Owns paths** (only edit inside these, plus tiny additive changes per AGENTS.md §4), **Soft deps** (use fakes until merged) and **Acceptance**. PRD references are in brackets.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### T00 · Contracts & skeleton
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Goal**: everything every other task plugs into, compiling and green, so all other tasks can run in parallel.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: whole repo (initial).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**:
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - Cargo workspace with all crates from PRD §7.2, each compiling, with the module folders every later task will fill (e.g. `il-nodes/src/{trigger,ai,knowledge,logic,data,human,output}/`, `il-connectors/src/{oauth,gmail,sheets,youtube,slack,discord,telegram,bluesky,shopify,social_dry_run,openapi}/`, `il-media/src/{ops,gen}/`, `il-architect/src/{compile,layout,catalog,validate,testfix,ask,appkit}/`, `il-engine/src/triggers/`) so later tasks never edit `lib.rs` module lists.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - `il-core`: Flow, Node, Edge, Group, Item, FileRef, NodeManifest types, events (PRD §8.4), API DTOs, error shape, expression evaluation (minijinja) with tests.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - `il-sdk`: `NodeExecutor`, `TriggerNode`, `NodeCtx`, service traits (`Store`, `ModelGateway`, `FileStore`, `KnowledgeService`, `Notifier`, `MediaGen`, `Clock`, `FlowRunner`, `ArchitectService`, `AgentTool`), `inventory` registries (nodes, triggers, providers, media gens, route factories), SSRF-safe HTTP client, `testkit` (in-memory store, fake gateway with scripted responses, fake files, fake HTTP, `NodeHarness`).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - **Stub manifests for every P0 node** in PRD §5.5, each in the folder given by the PRD §5.5 placement table, registered, with real settings schemas and `NotImplemented` executors; `GET /api/nodes` returns the full catalog.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - `il-server` boots, serves `/healthz`, `/api/nodes`, and the embedded web build; config loading from `config/interlock.toml` + env.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - `xtask`: `dev`, `check`, `check-deps`, `gen-types`, `catalog`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - Web: SvelteKit SPA + TS strict + `@xyflow/svelte` installed; `src/lib/api` client + SSE helper; generated types; **fixture mode** (`VITE_FIXTURES=1`) serving `web/fixtures/` including one recorded build stream and one recorded run stream for F-A; placeholder routes for every screen in PRD §7.3; `tokens.css` with the design tokens.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - CI (GitHub Actions) on `windows-latest` and `ubuntu-latest` running `cargo xtask check`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
  - `AGENTS.md`, `CLAUDE.md`, `docs/PRD.md`, `docs/WORKBOARD.md`, `docs/FUTURE.md`, `LICENSE` (Apache-2.0), README skeleton, `config/interlock.example.toml`, Dockerfile skeleton, `.gitignore`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B01 · Store
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-store/**`, all `migrations/*.sql` except `*_kb_*` (owned by K01).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: none.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: rusqlite (bundled) async wrapper; WAL mode; migrations runner; repositories implementing `Store` for workspaces, users, API keys (hashed), flows + versions, runs, events (append + cursor read), node runs, approvals, schedules, webhooks, poll cursors, credentials (AES-256-GCM), builds, usage ledger; `FileStore` on disk with preview URLs; sqlite-vec loads (smoke test) and FTS5 is available; tests for each repository.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B02 · Model gateway
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-ai/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: none (fake provider already in testkit).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: Token Factory provider (OpenAI-compatible: chat, streaming, tools, JSON-schema output, embeddings); roles → model IDs from config; startup check against `GET /v1/models` with clear logs; transparent fallback to JSON-in-prompt + validate/repair when tools/schema unsupported; retries with backoff; per-call token and cost metering to the current run/build; transcription via a configurable provider trait; wiremock tests; a `cargo run -p il-ai --example ping` live smoke test (needs `NEBIUS_API_KEY`).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B03 · Engine
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-engine/**` except `src/triggers/`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B01 (use testkit in-memory store).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [EN-1…EN-4, EN-6…EN-8]: executor over the flow graph (per-item/batch, named branch outputs, merges, parallel branches with concurrency limit); expression rendering per item; retries, timeouts, cancellation; Human-node pause → persisted → resume on decision (survives restart); spend caps; ordered event emission to the store and a broadcast bus; run replay from stored events; tests covering all of these with fake nodes.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B04 · Server, API, SSE, auth, demo
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-server/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B01, B03 (use testkit implementations until merged).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [PRD §8.5, WS-1, WS-2]: all internal `/api` routes wired to traits; SSE for runs and builds with `after` cursor and heartbeat; auto-mount of registered route factories (publish, oauth, hooks); GitHub OAuth sign-in with cookie sessions; demo sessions (ephemeral workspace, caps, seeded templates); multipart upload; consistent error shape; CORS; compression; rate limiting on public endpoints; integration tests with an in-memory setup.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B05 · Triggers & scheduler
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-engine/src/triggers/**`, `crates/il-nodes/src/trigger/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B03, B04.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [EN-5]: trigger supervisor that starts/stops triggers when flows activate/deactivate; Manual, Schedule (cron + timezone), Webhook (token URL → run), polling framework (interval + cursor) used by app triggers, Chat/API/MCP invocation entry point returning the Respond node's output; tests with a fake clock.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B06 · Nodes: logic & data
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-nodes/src/logic/**`, `crates/il-nodes/src/data/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: none.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: If, Switch (dynamic outputs), Loop over items (split a list field into items), Merge (append/by key/wait all), Wait (duration/until), Stop; Set fields, Template, HTTP request (SSRF-safe, auth options, JSON/text/binary), Parse document (delegates to the knowledge parsers trait, fake until K01); each with manifest examples and harness tests.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B07 · Nodes: AI
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-nodes/src/ai/**`, `prompts/nodes/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B02 (fake gateway).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: Write, Summarise, Classify (labels → named outputs), Extract fields (JSON schema output), Translate, Decide (yes/no or labels), Agent (tool loop: knowledge search tool, node-provided tools, OpenAPI tools via trait; max steps; returns answer + sources + activity), Describe image, Transcribe audio; all use roles (no model IDs); prompts in `prompts/nodes/`; snapshot tests of prompts; harness tests with scripted fake responses.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B08 · Nodes: human & output
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-nodes/src/human/**`, `crates/il-nodes/src/output/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B03.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: Approval (in-app; channel adapters via `Notifier` trait — Slack adapter lives in C02), with Approved/Rejected outputs, editable payload, timeout behaviour; Ask a question; Respond (for chat/API/MCP flows, streaming-capable); Send notification; harness tests.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B03b · Engine: approval expiry and timeout policy
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-engine/src/approvals/**`, `crates/il-engine/src/executor/**`; small additive approval contract and store changes as needed.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Hard deps**: B03.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: persist the approval deadline and `onTimeout` policy; after the deadline, approve, reject or keep waiting as configured; resume the correct run and item after a restart; reject late manual decisions; tests use a fake clock and store with no network.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### K01 · Knowledge
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-knowledge/**`, `crates/il-nodes/src/knowledge/**`, `migrations/*_kb_*.sql`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B01, B02.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [KN-1…KN-5]: parsers (PDF text layer, DOCX, MD, TXT, CSV, HTML); Tavily crawl/extract client (cap 50 pages, respects limits); transcription → Knowledge or Instructions; chunking with title/section/date metadata; embeddings via gateway; sqlite-vec + FTS5 hybrid search with reciprocal-rank fusion; `KnowledgeService` impl; nodes Knowledge base, Search knowledge, Add to knowledge; tests with fixtures (no network).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### K02 · Use any API (OpenAPI → tools)
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-connectors/src/openapi/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: none.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: parse OpenAPI 3.0/3.1 (URL or file) and cURL examples; list operations with risk flags (DELETE/destructive → skipped unless allowed); endpoint selection helper for the architect; "Use any API" node (pick operation, map params, auth via credential); tool adapter so the Agent node can call selected operations; tests with sample specs in `fixtures/apis/`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### P01 · Publish
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-publish/**`, `web/widget/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B03, B04.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [PB-1…PB-6]: `POST /v1/chat/completions` (stream + non-stream) and `GET /v1/models` backed by published AI-app flows; per-flow MCP (rmcp, streamable HTTP) with `ask_<slug>`, search and tool exposure; platform MCP (`list_flows`, `get_integration_guide`, `add_documents`, `create_flow_from_goal`, `run_flow`); widget bundle (vanilla TS, tiny, streams with sources); `INTERLOCK.md` generator with cURL/JS/Python/Rust; API keys (create/reveal once/revoke); tests including the official OpenAI JS SDK against a local server.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### C01 · OAuth broker + Google
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-connectors/src/{oauth,gmail,sheets,youtube}/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B01, B04, B05.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: generic OAuth2 broker (start/callback/refresh, encrypted tokens, provider configs self-registered); Gmail: new-email trigger (polling with cursor), reply, send, label; Sheets: append row; YouTube: upload (private); all with dedupe keys and harness tests using wiremock.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### C02 · Messaging
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-connectors/src/{slack,discord,telegram,bluesky}/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B08.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: Slack (OAuth or bot token): send message, approval with interactive buttons → decision callback route; Discord: webhook/bot send with attachments; Telegram: bot send with media; Bluesky: post with video/image via app password; `Notifier` adapters; tests with wiremock.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### C03 · Shopify sandbox + social dry-run
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-connectors/src/{shopify,social_dry_run}/**`, `fixtures/sandbox-store/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: none.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: built-in sandbox store (fictional products with images) behind the same node interface as real Shopify (real Shopify via Admin API token optional); "New product" trigger and "Get products"; Instagram, TikTok, LinkedIn, X nodes with full manifests, input validation, a rendered post preview in the output, `status: DryRun`, and a clear note "Needs platform approval to post"; tests.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### M01 · Media ops
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-media/src/ops/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: none.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: ffmpeg wrappers (child process, progress parsing, timeouts): cut clips, resize for platform (9:16, 1:1, 16:9 with smart crop/pad), add captions (burned in from text/SRT), merge audio and video, thumbnail, convert; nodes registered; tests with tiny fixture media (skipped if ffmpeg is missing, with a clear message).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### M02 · Media generation
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-media/src/gen/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: none.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: `MediaGen` adapters for video, voice and image, each written to its provider's documented API and switched on only when that provider's key is configured (one real adapter each, chosen in config). **Live-provider testing is out of scope for the hackathon**: the build only shows that real generation is possible. A `recorded` provider returning fixture media is the default in demo mode, in test runs and in tests. Nodes Generate video, Generate voice, Generate image with progress events and cost reporting; adapter tests against wiremock only.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### A01 · Architect: goal → flow
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-architect/src/{compile,layout,catalog,validate}/**`, `prompts/architect/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: B02 (scripted fake model).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [AR-1, AR-3…AR-5, AR-7, AR-10]: builds a compact catalog from registered manifests; compiles goal + attachments into Flow JSON with the Architect role; validates (schema, known types, settings schemas, edges/ports, expressions reference real nodes); repair loop with errors (max 3); clarifying questions when required values are missing (AR-2, P1); layered auto-layout with groups; connection check; emits build events incrementally (`build.flow_patch` per node/edge); golden tests for F-A, F-B, F-C goals with scripted responses; a live example run behind a flag.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### A02 · Architect: test-and-fix, Ask diff, app kit
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-architect/src/{testfix,ask,appkit}/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Hard deps**: A01. **Soft deps**: B03.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [AR-6, AR-8, AR-9]: test run with sample data through the engine trait, failure analysis, up to 2 fix rounds with `build.fix` events; Ask Interlock → JSON-Patch diff with summary ("+1 step, 1 changed"); for AI apps, generate guardrails and a test suite (knowledge questions + tool tasks), run it, and fix prompts on failure; tests.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W01 · Design system & app shell
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/lib/design/**`, `web/src/routes/+layout.svelte`, `web/src/app.css`, `web/src/app.html`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: Claude Design export in `design/`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: tokens and every base component from the design (Button, IconButton, Input, TextArea, Select, SegmentedControl, Toggle, Checkbox, Chip, StatusPill, KindBadge, Tabs, Tooltip, Popover, Menu, Modal, BottomSheet, Toast, Banner, AppTile, PartnerSlot, BuiltWithStrip, EmptyState, Skeleton, CopyField, CodeTabs), icon set, sidebar + top bar + mobile tab bar, light theme (dark P1), a `/dev/components` gallery page; vitest for interactive components.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W02 · Canvas editor + Ask diff view
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/lib/canvas/**`, `web/src/routes/flows/[id]/+page.svelte`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: W01, W03 (fixture mode).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [ED-1, ED-2, ED-6, ED-9]: Svelte Flow canvas with NodeCard (all states from the design), edges with running animation and counts, groups, minimap, toolbar (zoom, fit, tidy via API, undo/redo), palette from `/api/nodes` with search and drag-to-add, insert on edge, Replace with… (keeps compatible settings/mappings), list view, Ask Interlock bar + diff rendering (added/changed/removed) with accept/discard; renders live state from the run SSE stream; works fully in fixture mode.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W03 · Node panel, schema forms, previews
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/lib/panel/**`, `web/src/lib/forms/**`, `web/src/lib/previews/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: W01.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [ED-3…ED-5, ED-7]: JSON-Schema form renderer (string, number, boolean, enum, array, object, oneOf) with `x-widget` hints (expression, credential, file, longtext, select, model-role); field mapper chips with an expression picker listing upstream outputs; credential picker with "Connect"; Input/Output tabs with preview renderers (text, email, image, video, audio, table, JSON) auto-discovered; "Run this step"; vitest coverage for the form renderer.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W04 · Create & Building
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/routes/new/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: W01, W02.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [AR-1, AR-7]: prompt box with example chips, attachments (files, links, audio) with Knowledge/Instructions toggles, voice input (browser speech recognition where available; otherwise record and upload for transcription), detected apps row, clarifying questions card; Building screen rendering the canvas incrementally from build SSE events, stepper, log, cost/time; "Open editor" on ready; works from the recorded build fixture.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W05 · Runs, approvals, flows, templates, connections
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/routes/flows/+page.svelte`, `web/src/routes/flows/[id]/runs/**`, `web/src/routes/approvals/**`, `web/src/routes/templates/**`, `web/src/routes/connections/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: W01, W02, W03.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: run view (canvas read-only + step list + clip gallery + approval card), approvals inbox (mobile-first), flows list with filters and sparklines, templates gallery with "Use template", connections grid with OAuth start and detail drawer; mobile layouts per design.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W06 · Publish/Connect & chat
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/routes/flows/[id]/publish/**`, `web/src/routes/flows/[id]/chat/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: W01.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [PB-*]: Activate state for automations; Publish page for AI apps with API, MCP and widget cards, reveal-once key, quick-start code tabs, integration guide preview and copy actions, platform MCP section; chat playground with streaming, activity chips, source chips with passage preview, suggested actions, "Try asking", and path highlight.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W07 · Landing, demo, settings, states
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/routes/+page.svelte`, `web/src/routes/demo/**`, `web/src/routes/settings/**`, `web/src/routes/+error.svelte`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: W01.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [MK-1, WS-2, WS-5]: landing page per design (desktop + mobile); demo entry creating a demo session and banner; settings (workspace, API keys, usage and caps); 404, error, offline banner, cap-reached states.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### R01 · Templates, fixtures, recorded runs, demo seed
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `fixtures/templates/**`, `fixtures/recorded/**`, `fixtures/docs/**`, `fixtures/apis/**`, `web/fixtures/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Soft deps**: node stubs from T00 are enough to author templates.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [MK-2, MK-3]: 10 templates as Flow JSON validated against the catalog (F-A, F-B, F-C + 7 from the design list); sample documents, a short sample audio memo, the Larkspur OpenAPI spec and a tiny mock API; recorded event streams + media for F-A and F-C; demo seeding on first boot; a validation test that loads every template.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### R02 · Deploy to Nebius
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `deploy/**`, `Dockerfile`, `.github/workflows/deploy.yml`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Hard deps**: B04.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: multi-stage image (web build → Rust release → slim runtime with ffmpeg, non-root, `/data` volume); deployed on Nebius with HTTPS and a persistent volume; secrets set; global spend cap on; public access for judges confirmed (no token wall); `deploy/README.md` runbook; smoke test script hitting `/healthz`, `/api/nodes`, demo session and a template run.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### R03 · Integration pass & E2E
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/e2e/**`, `tests/e2e/**`; cross-cutting fixes by agreement (note each in the PR).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Hard deps**: B03, B04, A01, W02, W03.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: F-A, F-B, F-C run end to end against real services on a staging deploy; fixture mode turned off in production builds; Playwright smoke (create → building → editor → test run → publish → chat) green in CI using fake providers; a bug list turned into new task rows.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### W01b · Remaining design components + gallery
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `web/src/lib/design/**`, `web/src/routes/dev/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: the W01 components not yet built (Checkbox, KindBadge, Tooltip, Menu, EmptyState, Skeleton, CodeTabs, DropZone, FileRow with the Knowledge/Instructions toggle, ApiCard), matching `design/interlock/screens/SpecV3.png`; a `/dev/components` gallery showing every state; vitest for interactive components.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### N01 · Node: ai.guardrails
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-nodes/src/ai/guardrails.rs` (+ its line in `ai/mod.rs`).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: manifest (rules array, hand-off rules such as "refunds over $100", outputs `pass`/`blocked`), harness tests with the fake gateway, and `cargo xtask catalog` regenerated. The architect uses it for F-C (AR-9). The design's "Safety rules" node maps to it (see `design/interlock/MAPPING.md`).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### B04b · Server: GitHub sign-in, rate limits, startup recovery, demo seed
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `crates/il-server/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: GitHub OAuth sign-in (`/api/auth/github/start`, callback, creates a `Session` in the store like `POST /api/demo/session` does, cookie `il_session`); per-IP rate limiting on `/hooks`, `/v1`, `/mcp`, `/api/demo/session`; startup recovery (`engine.recover` for runs left Running, `triggers.restore` for active flows: add the store queries this needs with B01); demo sessions seeded with templates once R01 lands; integration tests.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### T01 · Build size: lighter dev/test debug info
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: root `Cargo.toml` profiles.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance**: dev and test builds keep only line-table debug info for workspace crates and none for dependencies, so panics and backtraces still show file and line; `target` shrinks substantially on Windows; `cargo xtask check` and CI pass. CI already runs without incremental builds (`dtolnay/rust-toolchain` sets `CARGO_INCREMENTAL=0`).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
### R04 · README, docs, video script, Devpost
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Owns**: `README.md`, `docs/**` (except PRD and WORKBOARD), `assets/**`.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Acceptance** [PRD §13]: README with what/why, demo link, screenshots, architecture diagram, setup for Windows and Linux, how Nemotron Ultra/Super/Nano and Token Factory are used (with file paths), Tavily usage, Nebius deployment, honest limitations; `docs/NODES.md` generated; a demo video script under 3 minutes; Devpost text including the Tilo lineage note and feedback on Nebius/NVIDIA tools.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
- **Licence (owner decision, 2026-10-27)**: the repo is public under an all-rights-reserved `LICENSE` for now. The submission checklist (PRD §13) asks for Apache-2.0 visible in the About section, so switch `LICENSE`, `Cargo.toml`, `web/package.json` and the README back to Apache-2.0 at submission (owner's decision). The root `README.md` was removed on 2026-10-27 so the public repo page stays bare (owner's decision); write it again from scratch for submission (its last version is in git history, commit 7c53926).
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
---
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
## Suggested parallel plan (not binding — self-claiming still applies)
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| When (IST) | claude-a (Windows, local) | claude-b (local worktree) | codex-1 | codex-2 |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
|---|---|---|---|---|
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Tue morning | T00 | — (design export into `design/`) | — | — |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Tue afternoon–night | B03 → B05 | W01 → W02 | B02 → B07 | B01 → B06 |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Wed | B04 → B08 | W03 → W04 | A01 → A02 | K01 → K02 → R01 |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Thu | R03 (integrator) + R02 prep | W05 → W06 → W07 | P01 → C02 | C01 → M01 → M02 → C03 |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
| Fri | R03 → R02 deploy | polish + mobile | fixes from R03 | R04 |
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
Keep one agent on the critical path at all times: **T00 → B03 → B04 → R03 → R02**.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |

| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
Good fits for cloud agents (well-specified, self-contained): B01, B02, B06, B07, K01, K02, C01–C03, M01, M02, P01, R01. Good fits for local agents (visual checks, Windows build, integration): T00, W01–W07, B04, R02, R03.
| 36 | C04 | API-key connections: save keys and tokens from /connections | P1 | C01 | ready | | | | `POST /api/connections` (app, label, secret; encrypted via the secret store) and a key form in the /connections drawer for bot tokens, app passwords, webhooks and API keys |
