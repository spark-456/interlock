# AGENTS.md — rules for every coding agent on Interlock

This file is read by **Codex** (AGENTS.md) and **Claude Code** (via `CLAUDE.md`, which imports this file). Every agent, human or AI, follows the same rules so that anyone who is free can pick up any ready task without blocking anyone else.

**Read first, every session:** `docs/PRD.md` (what and how), `docs/WORKBOARD.md` (what to do next), then the task card you claim. If these disagree, the PRD wins; raise the conflict in your PR.

---

## 1. Setup

### Windows (PowerShell, native)

```powershell
winget install Rustlang.Rustup; rustup default stable-msvc
winget install Microsoft.VisualStudio.2022.BuildTools   # select "Desktop development with C++"
winget install OpenJS.NodeJS.LTS; corepack enable; corepack prepare pnpm@latest --activate
winget install Gyan.FFmpeg
git clone <repo-url> interlock; cd interlock
pnpm -C web install
cargo xtask check
```

### Linux / macOS / Codex cloud

```sh
curl --proto '=https' -sSf https://sh.rustup.rs | sh -s -- -y
# Node 22+ with corepack/pnpm, ffmpeg from the package manager
pnpm -C web install
cargo xtask check
```

### Environment

Copy `config/interlock.example.toml` to `config/interlock.toml`. Secrets only via env: `NEBIUS_API_KEY`, `TAVILY_API_KEY`, `IL_MASTER_KEY` (32-byte base64), `GITHUB_CLIENT_ID/SECRET`, `GOOGLE_CLIENT_ID/SECRET`, `SLACK_CLIENT_ID/SECRET`, media provider keys. **Tests never need network or keys:** they run with `IL_PROVIDERS=fake`.

## 2. Commands

| Command | Does |
|---|---|
| `cargo xtask dev` | Runs the server (port 8080) and the web dev server with API proxy |
| `cargo xtask check` | fmt check, clippy `-D warnings`, all Rust tests, dependency-direction check, TS type drift check, web check + tests |
| `cargo xtask gen-types` | Regenerates `web/src/lib/api/types/` and `contracts/*.json` from Rust |
| `cargo xtask catalog` | Prints/updates `docs/NODES.md` from registered manifests |
| `pnpm -C web dev` | Web only; add `VITE_FIXTURES=1` to run with no backend |
| `pnpm -C web check` / `pnpm -C web test` | Svelte/TS check, vitest |
| `pnpm -C web e2e` | Playwright smoke tests |
| `cargo run -p il-server` | The single binary (serves the embedded web build) |

Scripts must work on both Windows and Linux: use `cargo xtask` or `pnpm` scripts, never bash-only scripts.

## 3. Work protocol (claim → build → PR → done)

1. `git pull --rebase origin main`
2. Open `docs/WORKBOARD.md`. If a human gave you a task ID, take it. Otherwise take the **first task in the priority list** whose status is `ready` and whose **Hard deps** are all `done`.
3. **Claim it**: edit only that task's row in the index table — Status `claimed`, Owner = your agent name (`claude-a`, `claude-b`, `codex-1`, `codex-2`, `human`), Branch `t/<ID>-<slug>`, Updated = UTC time. Commit `chore(board): claim <ID>` and push to `main`.
   - If the push is rejected, pull with rebase. If someone else claimed it, pick another task.
   - **Cloud agents that cannot push to main** (e.g. Codex cloud): the human marks the claim when launching you. Put the board update for "review/done" inside your PR.
4. Work on a branch, preferably in a separate worktree: `git worktree add ../il-<ID> -b t/<ID>-<slug>`.
5. Commit often; push your branch at least every hour (pushed commits are how others see a claim is alive).
6. When the task card's acceptance criteria and the global definition of done pass, rebase on `main`, open a PR titled `[<ID>] <title>`, and set Status `review`.
7. After merge (squash), set Status `done` with the PR link. The integrator (or you) merges when CI is green.
8. **Stale claims**: a claim with no pushed commits for 3 hours may be taken over. Write "taken over from <owner>" in Notes.
9. **Ending a session mid-task**: push, then update the task's Notes with what is done and what is left, so any agent can continue.

Only `docs/WORKBOARD.md` edits may be pushed directly to `main`. Everything else goes through a PR (or, for a human working alone, a local merge after `cargo xtask check` passes).

## 4. Never block, never deadlock

- **Soft deps are not blockers.** If your task needs something another task is still building, use the fake or fixture from `il-sdk::testkit` or `web/fixtures/`, code against the trait or contract, and keep going.
- **Stub manifests exist for every P0 node** (created in T00). Templates, the architect and the UI can use any node type ID from day one; implementations replace `NotImplemented` executors later.
- **Only take work inside your task's "Owns paths".** If you need a small change elsewhere:
  - additive and tiny (a new optional field, a new dependency line, a new event type): do it in your PR and mention it;
  - anything bigger: add a new task row (status `ready`) describing it, and continue with a fake.
- **Truly blocked** (a hard dependency is broken on `main`): set Status `blocked` with the reason, then claim another task. Never wait.
- **Keep PRs small and frequent** (aim for under ~800 changed lines). Merge early behind sensible defaults rather than holding a big branch.

## 5. Modularity rules (hard rules — CI enforces several)

1. **Dependency direction**: `il-core ← il-sdk ← implementations`. `il-engine`, `il-architect` and `il-publish` use only traits from `il-sdk`. Only `il-server` wires concrete crates together. `cargo xtask check-deps` fails on violations.
2. **No central lists.** Nodes, triggers, providers, media generators, connectors and route groups self-register with `inventory::submit!`. Web previews and icons are discovered with `import.meta.glob`. Screens use file-system routes. Never add a hand-maintained registry.
3. **One unit, one folder.** A node lives in its own module with its manifest, executor and tests. A connector lives in `il-connectors/src/<app>/`. A preview renderer is one `.svelte` file.
4. **The manifest is the interface.** The palette, settings form, architect catalog and `docs/NODES.md` all come from `NodeManifest`. Never write per-node UI code for settings; use JSON Schema with `x-widget` hints. Custom UI is allowed only for output previews.
5. **Contracts live in `il-core`.** TypeScript types are generated (`cargo xtask gen-types`) — never edit `web/src/lib/api/types/` by hand.
6. **Model IDs only in config.** Code asks for a role (`Architect`, `Agent`, `Fast`) through `ModelGateway`.
7. **Side effects go through `NodeCtx`** (HTTP via the SSRF-safe client, files via `FileStore`, credentials via `ctx.credential()`), never directly.
8. **Migrations** are timestamped and prefixed by module: `migrations/<YYYYMMDDHHMM>_<module>_<desc>.sql`. Tables are prefixed by module (`kb_`, `rn_`, `fl_`…). Never edit a merged migration.
9. **Config**: each module reads its own section of `interlock.toml`; add new sections, don't restructure others.
10. **Web styling**: colours, spacing, radii and shadows come from `src/lib/design/tokens.css`. No hex values or one-off styles outside the design folder.

## 6. Contract changes

- **Additive** (new optional field, new event type, new endpoint, new enum variant the UI can ignore): allowed in any task. Run `cargo xtask gen-types`, commit the generated files, and note it in the PR under "Contract changes".
- **Breaking** (rename, remove, type change, changed semantics): open a dedicated PR titled `contract: <change>` that updates `il-core`, the generated types and every call site in one go, and add a line to the Contract log in `docs/WORKBOARD.md`. Keep it small, merge it fast, and tell other agents to rebase.

## 7. Coding standards

**Rust**
- Libraries: `thiserror` error enums; `anyhow` only in `il-server` and `xtask`. No `unwrap()`/`expect()` outside tests and startup.
- Async everywhere on tokio. CPU-heavy or blocking work goes in `spawn_blocking` or a child process.
- `tracing` for logs, with `run_id`, `node_id`, `flow_id` fields. Never log secrets, tokens, email bodies or file contents.
- Public items get a one-line doc comment. Keep modules under ~400 lines; split when larger.

**Web**
- Web tasks must match `design/interlock/` (the visual source of truth) and its `MAPPING.md`, which maps each design screen to its PRD route and task; the PRD wins on routes, data, API and events.
- Svelte 5 runes, TypeScript strict, no `any` in app code.
- Components are small and live in the owning module's folder. New npm dependencies must be justified in the PR (bundle budget: first screen < 150 KB gzipped).
- All server calls go through `src/lib/api/`; all live updates through the SSE helper.
- Accessibility: labelled controls, visible focus, keyboard reachable.

**Commits and PRs**
- Conventional commits: `feat(il-nodes): add ai.classify`, `fix(web/canvas): …`, `chore(board): claim B06`.
- PR description: task ID, what changed, how it was tested, contract changes, follow-ups (new task rows added).

## 8. Testing rules

- Every node has tests using `il_sdk::testkit::NodeHarness` (fake gateway, fake files, fake HTTP).
- Every crate's tests run offline. Use `wiremock` for HTTP providers, recorded fixtures for model outputs, and `insta` snapshots for schemas and generated prompts.
- The engine has tests for branches, merges, per-item vs batch, retries, cancellation, approvals and caps.
- The architect has golden tests: fixed goals + scripted fake model responses → a valid flow (schema-valid, known node types, required settings present).
- Web: vitest for logic (forms, mappers, stores); Playwright smoke in fixture mode for: create → building → editor → test run → publish.

## 9. Security rules

- Credentials and OAuth tokens are encrypted at rest and never returned to the browser or written to logs.
- Destinations, recipients and credentials come only from node settings/credentials, never from item data, unless the user explicitly mapped them.
- All outbound HTTP uses the SSRF-safe client (blocks private/loopback/link-local ranges; re-checks redirects).
- Webhooks use unguessable tokens; API keys are stored hashed; demo workspaces are isolated and capped.
- Never commit secrets. `config/interlock.toml` is git-ignored.

## 10. Definition of done (every task)

- [ ] Acceptance criteria on the task card are met
- [ ] Tests added and passing offline
- [ ] `cargo xtask check` passes locally (includes fmt, clippy, tests, deps, type drift, web check/tests)
- [ ] Docs updated where behaviour changed (`docs/NODES.md` regenerated if manifests changed)
- [ ] PR open with the description template; WORKBOARD row updated

## 11. When in doubt

Choose the simpler design that keeps modules independent, write it down in the PR, and keep moving. Ask the human only for things that are genuinely theirs to decide (product scope, credentials, accounts, money).
