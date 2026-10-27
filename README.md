# Interlock

**Describe it. Interlock builds it. You can change anything.**

Interlock is an AI automation and backend builder. You describe what you want, in text, by voice or with documents and audio. Interlock builds it as a working flow on a visual canvas, made of triggers, AI steps, knowledge, media, logic, app integrations, human approvals and outputs. Every flow is editable node by node, can be tested, and can be published as an automation, an OpenAI-compatible API, an MCP server or a chat widget.

Built for the Nebius × NVIDIA Global AI Hackathon (Oct 2026). Interlock builds on ideas from the team's earlier project Tilo, with a new codebase, design and feature set.

> Status: hackathon build in progress. See `docs/PRD.md` and `docs/WORKBOARD.md`.

## Run it

Prerequisites: Rust stable (MSVC on Windows), Node 22+ with pnpm (via corepack), and ffmpeg. See `AGENTS.md` §1.

```sh
pnpm -C web install
cargo xtask check          # fmt, clippy, tests, dependency check, type drift, web check + tests
cargo xtask dev            # server on :8080 + web dev server with API proxy
pnpm -C web dev:fixtures   # web only, no backend: every API call and SSE stream comes from web/fixtures/
pnpm -C web build && cargo run -p il-server   # the single binary serving the embedded web app
```

Configuration: copy `config/interlock.example.toml` to `config/interlock.toml`. Secrets are read only from environment variables. Tests run offline with `IL_PROVIDERS=fake`.

## Architecture

The backend is a Rust workspace (`crates/`). `il-core` holds the contracts, `il-sdk` the traits, registries and testkit, and implementation crates plug into them. The web app is a SvelteKit SPA (`web/`). Nodes, providers and route groups register themselves, so adding one touches only its own folder. See PRD §7 and §9.

| Where NVIDIA Nemotron models are used | Role |
|---|---|
| Nemotron 3 Ultra | `architect`: goal → flow, Ask Interlock diffs |
| Nemotron 3 Super | `agent`: agents, writing |
| Nemotron 3 Nano | `fast`: classify, extract, route |

All models are served through Nebius Token Factory. Model IDs live only in config.

## Licence

Apache-2.0. See `LICENSE`.
