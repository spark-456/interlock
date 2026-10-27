# KICKOFF — how to start and run the build with Claude Code and Codex

## 1. Before the first agent starts (human, ~30 min)

1. **Create the GitHub repo** `interlock` (private until submission is fine; it must be public with the licence visible by the deadline).
2. **Unzip this bundle into the repo root**, so you have `AGENTS.md`, `CLAUDE.md`, `docs/PRD.md`, `docs/WORKBOARD.md`, `docs/KICKOFF.md`, `docs/FUTURE.md`.
3. **Add the design**:
   - copy the Claude Design prompt (project doc `claude-design-prompt-interlock.md`) to `docs/DESIGN-BRIEF.md` — it holds the sample data the fixtures use;
   - export the Claude Design artboards (HTML/CSS if available, otherwise PNG screenshots of every P0 artboard, plus the spec and brand sheets) into `design/`.
4. **Commit and push** to `main`.
5. **Get keys** (they go in env vars, never in the repo):
   - `NEBIUS_API_KEY` (Token Factory; Builder Program credits) and `TAVILY_API_KEY`
   - GitHub OAuth app (sign-in) → `GITHUB_CLIENT_ID/SECRET`
   - Google Cloud OAuth client in testing mode with Gmail, Sheets and YouTube scopes; add yourself and test accounts as test users → `GOOGLE_CLIENT_ID/SECRET`
   - Slack app (bot token + interactivity URL), a Discord webhook, a Telegram bot token, a Bluesky app password
   - Media generation keys are **not needed**: the recorded provider is the default everywhere and the real adapters are not live-tested
   - `IL_MASTER_KEY`: generate 32 random bytes, base64
6. **Codex setup**:
   - Codex CLI (local): run it inside its own git worktree; it reads `AGENTS.md` automatically.
   - Codex cloud: connect the repo and set the environment setup script to:
     ```sh
     rustup default stable && rustup component add clippy rustfmt
     corepack enable && pnpm -C web install
     (apt-get update && apt-get install -y ffmpeg) || true
     ```
     Tests run offline, so cloud tasks don't need keys.
7. **Claude Code setup**: it reads `CLAUDE.md`, which imports `AGENTS.md`. Use one git worktree per parallel session.

## 2. Order of work

1. **One agent runs T00** (Prompt A below). Recommended: Claude Code on the Windows PC, because T00 must prove the Windows build.
2. As soon as T00 is merged, **start as many agents as you have** (Prompt B). Every task except A02, R02 and R03 depends only on T00, and each agent picks the next ready task from the board itself.
3. From Thursday, **one agent acts as integrator** (Prompt D) a few times a day, keeping `main` green and turning bugs into new tasks.
4. Friday: R02 (deploy) and R04 (README, video, Devpost); submit by 6 PM IST, well before the 10:30 PM IST deadline.

Tips for you as the dispatcher:
- Keep one agent on the critical path at all times: T00 → B03 → B04 → R03 → R02.
- When an agent finishes, just send it Prompt B again (or "continue with the next ready task").
- For Codex cloud tasks, mark the claim on the board yourself before launching (Prompt C).

---

## Prompt A — Bootstrap (T00), paste into Claude Code

```
You are bootstrapping the Interlock repository. Read AGENTS.md, docs/PRD.md (all of it), docs/WORKBOARD.md (the T00 card) and docs/DESIGN-BRIEF.md (for sample data and design tokens).

Your job is task T00: create the contracts and skeleton that every other task plugs into, so that after you finish, up to four agents can work in parallel without touching the same files. Claim T00 on the board first (AGENTS.md §3).

Work autonomously through the whole task; do not stop to ask for approval between steps. Deliver all of the following:

1. Cargo workspace with every crate in PRD §7.2 (il-core, il-sdk, il-store, il-ai, il-engine, il-nodes, il-connectors, il-media, il-knowledge, il-architect, il-publish, il-server) plus an `xtask` crate. Create every module folder named in the T00 card with a stub `mod.rs`, so later tasks never edit `lib.rs` module lists.
2. il-core: all contract types from PRD §8 (Flow, Node, Edge, Group, Item, FileRef, NodeManifest and friends, run and build events, API DTOs, error shape), serde + schemars + ts-rs derives, and minijinja-based expression rendering with tests.
3. il-sdk: NodeExecutor, TriggerNode, NodeCtx, the service traits (Store, ModelGateway, FileStore, KnowledgeService, Notifier, MediaGen, Clock, FlowRunner, ArchitectService, AgentTool — see PRD §7.2), inventory registries (nodes, triggers, providers, media generators, route factories), the SSRF-safe HTTP client, and `testkit` (in-memory Store, scripted fake ModelGateway, fake FileStore, fake HTTP, NodeHarness).
4. Stub manifests for EVERY P0 node in PRD §5.5, each in its own module in the crate folder given by the PRD §5.5 placement table, registered via inventory, with a real settings JSON Schema (x-widget hints), ports, category, credentials and 1–2 examples, and an executor that returns NodeError::NotImplemented. Use type IDs like `trigger.schedule`, `ai.write`, `logic.switch`, `data.http`, `knowledge.search`, `media.cut_clips`, `human.approval`, `output.respond`, `app.gmail.reply`, `app.instagram.post_reel`.
5. il-server: boots from config/interlock.toml + env, serves /healthz, GET /api/nodes (full catalog from the registry), mounts registered route factories, and serves the embedded web build (rust-embed) with SPA fallback.
6. xtask: `dev`, `check` (fmt, clippy -D warnings, tests, check-deps, type drift, web check + tests), `check-deps` (enforces PRD §7.2 dependency direction using cargo metadata), `gen-types` (ts-rs → web/src/lib/api/types, JSON Schemas → contracts/), `catalog` (writes docs/NODES.md).
7. web/: SvelteKit (Svelte 5, TS strict, adapter-static SPA), pnpm, @xyflow/svelte installed; src/lib/design/tokens.css with every token from the design brief; src/lib/api client + SSE helper + generated types; FIXTURE MODE (VITE_FIXTURES=1) serving web/fixtures/*.json, including a hand-authored recorded build stream and a recorded run stream for flow F-A that use the PRD §8.4 event types and the design brief's sample data; placeholder routes for every screen in PRD §7.3 that render their name and use the app shell.
8. Repo files: LICENSE (Apache-2.0), README skeleton, config/interlock.example.toml (all sections from PRD §11, model roles mapped to the Nemotron Ultra/Super/Nano IDs and the Qwen3 embedding model, with a comment to verify IDs via GET /v1/models), .gitignore (config/interlock.toml, data/, target/, node_modules/), Dockerfile skeleton (multi-stage, ffmpeg in runtime), docs/NODES.md generated, migrations/ folder with a README explaining the naming rule.
9. CI: .github/workflows/ci.yml running `cargo xtask check` on windows-latest and ubuntu-latest, caching cargo and pnpm.

Definition of done: `cargo xtask check` passes on this Windows machine; `cargo run -p il-server` serves the web app and /api/nodes returns the full catalog; `pnpm -C web dev` with VITE_FIXTURES=1 shows the placeholder screens and replays the fixture streams in the console. Open the PR "[T00] Contracts & skeleton", merge it when green, mark T00 done on the board, and list in the PR description anything later tasks must know.
```

## Prompt B — Worker (any agent, any time)

```
You are a worker agent on Interlock. Follow AGENTS.md exactly.

1. git pull --rebase origin main. Read AGENTS.md, docs/PRD.md and docs/WORKBOARD.md. For web tasks also read docs/DESIGN-BRIEF.md and look at design/.
2. Task: {TASK_ID — or "pick the first ready task whose hard deps are done, per AGENTS.md §3"}. Claim it on the board and push the claim.
3. Implement the entire task card to its acceptance criteria, autonomously, without stopping for approval. Stay inside the card's Owns paths. Where a soft dependency isn't merged yet, code against the il-sdk traits and use il_sdk::testkit fakes or web fixtures; do not wait.
4. Keep modules self-registering (inventory / import.meta.glob); never add central lists, never hand-edit generated types, never hardcode model IDs.
5. Run `cargo xtask check` until it passes. Open a PR titled "[<ID>] <title>" with: what changed, how it was tested, contract changes (if any), follow-ups. Set the task to `review` on the board.
6. If you have context left, claim the next ready task and continue. If you must stop mid-task, push your branch and write handoff notes in the task row so any other agent can continue.
```

## Prompt C — Codex cloud addendum (append to Prompt B)

```
You are running in Codex cloud and cannot push to main. The human has already marked your claim for task {TASK_ID} on the board. Work on branch t/{TASK_ID}-<slug> and open a PR. Include the WORKBOARD row change (status → review, your PR link) inside the PR. Tests must pass offline (IL_PROVIDERS=fake); do not call external APIs.
```

## Prompt D — Integrator (Thursday onward, or whenever several PRs are waiting)

```
You are the integrator for Interlock. Follow AGENTS.md. Your job is to keep main green and moving; do not build large features yourself.

1. Pull main. List every task in `review` on docs/WORKBOARD.md and its PR/branch.
2. For each, oldest first: rebase on main, run `cargo xtask check`, resolve conflicts preserving both sides' intent, and squash-merge when green. Mark the task done with the PR link. If a PR is broken beyond a quick fix, comment what's wrong, set it back to `claimed` and move on.
3. If main is ever red, fix it immediately or revert the offending merge.
4. Run the app (`cargo xtask dev`) and walk through F-A, F-B and F-C (PRD §4) in fixture mode and, when keys are available, against real services. For every bug or gap, add a new task row (status ready, Owns paths, acceptance) instead of fixing large things yourself.
5. Check stale claims (no pushes for 3 hours) and note them so another agent can take over.
6. Post a short summary at the end of docs/WORKBOARD.md under "Integration notes" with the date: merged, broken, new tasks.
```

## Prompt E — Release (Friday)

```
You are preparing the Interlock submission. Follow AGENTS.md and complete R02 and R04 from docs/WORKBOARD.md, plus the PRD §13 compliance checklist:
- Deploy the image to Nebius with HTTPS, a persistent volume, secrets and the global spend cap; confirm judges can open the demo with no sign-in; run the smoke script.
- Seed demo content and recorded runs; confirm templates open instantly.
- Finalise README (what/why, demo link, screenshots, architecture, Windows and Linux setup, exactly where Nemotron Ultra/Super/Nano and Token Factory are used with file paths, Tavily usage, Nebius deployment, honest limitations), regenerate docs/NODES.md, make the repo public with the Apache-2.0 licence visible.
- Write docs/SUBMISSION.md with the Devpost text (features, track, Tilo lineage note, feedback on Token Factory, AI Cloud and NVIDIA models) and a shot-by-shot script for a video under 3 minutes.
Report anything in the checklist that is not done.
```
