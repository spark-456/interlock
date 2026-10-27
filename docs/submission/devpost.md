# Devpost text (draft)

Draft for the owner. Fill the bracketed parts from real use before submitting.

## Inspiration
Building an AI assistant or an automation that does real work still means stitching together models, retrieval, tool calls, prompts, guardrails and tests by hand, then wrapping it in an API. We wanted that whole backend to come from a sentence, stay inspectable, and be tested before it goes live.

Interlock builds on ideas from our earlier project, Tilo, with a new codebase, design and feature set.

## What it does
- **Describe it, and Interlock builds it.** Type or say a goal, drop in documents, links, voice memos and API specs, and mark each as knowledge or instructions. The architect turns that into a flow on a canvas: triggers, a knowledge base, routers, agents with tools, guardrails and outputs, with every prompt written.
- **It tests and fixes itself.** Each build is test-run with sample data. Failures go back to the architect, which fixes the flow and runs it again (up to two rounds). AI apps also get a generated test suite of knowledge questions and tool tasks, judged automatically.
- **Edit in place, or ask.** Every step has a settings form generated from its schema, live input and output previews, and a "Replace with…" picker. Ask Interlock turns "hand refunds over $100 to a person" into a diff you accept or discard.
- **Publish anywhere.** AI apps become an OpenAI-compatible API (streaming), a per-app MCP server, a one-line chat widget and an `INTERLOCK.md` integration guide; a platform MCP lets coding agents fetch the guide and wire it in. Automations run on schedules, webhooks and app events, with human approvals in-app or in Slack.
- **Try it without an account.** The demo has ten templates, recorded runs for the promo-video and support-agent flows, and capped live builds.

## How we built it
- **Rust** (axum, tokio) with one crate per module: core contracts, SDK traits, engine, nodes, connectors, media (ffmpeg), knowledge (SQLite with vector and full-text search), architect and publish. Nodes, providers and routes self-register, and the palette, settings forms, architect catalog and node docs all come from each node's manifest.
- **NVIDIA Nemotron on Nebius Token Factory**, by role:
  - **Nemotron 3 Ultra** — the architect: goal → flow, repairs, Ask Interlock diffs (`crates/il-architect`).
  - **Nemotron 3 Super** — agents and writing steps (`crates/il-nodes/src/ai/agent.rs`, `write.rs`).
  - **Nemotron 3 Nano** — fast steps: routing, classification, extraction, guardrails (by default) and the test-suite judge (`decide.rs`, `classify.rs`, `extract.rs`, `guardrails.rs`, `crates/il-architect/src/appkit`).
  - Model ids live only in config; code asks for a role through one gateway that meters cost and enforces spending caps.
- **Tavily** reads websites into the knowledge base (`crates/il-knowledge`).
- **SvelteKit + Svelte 5** front end with Svelte Flow for the canvas, server-sent events for live builds and runs, and a fixture mode so the UI runs with no backend.
- **One binary** serves the API and the embedded web app; it deploys as a single Docker image on Nebius.

## Challenges
- Getting a model to produce *valid* flows: we give it the live node catalog, validate every draft against each node's JSON Schema, and feed the errors back for up to three repairs.
- Making builds feel alive: the canvas fills in from real build events, not a fake timer.
- Keeping side effects safe: test runs never post or send for real, outbound HTTP goes through an SSRF-safe client, credentials are encrypted at rest, and risky API operations are skipped unless allowed.

## Accomplishments
- The three hero flows (promo videos, support inbox, support agent) build from their sentences.
- Self-testing builds with automatic fixes, and AI apps that ship as an API, an MCP server and a widget from the same flow.
- A design system built from the start, with a component gallery, mobile layouts and accessible controls.

## What we learned
[Owner: two or three honest lessons.]

## What's next
- Agents declaring API tools directly (see board task R05).
- Usage dashboards and editable caps (WS-5).
- More connectors and templates.

## Feedback on Nebius Token Factory, AI Cloud and NVIDIA models
[Owner: real notes from building — latency, structured output and tool calling with Nemotron, rate limits, deployment on Nebius, what was smooth and what wasn't.]

## Built with
Rust, axum, tokio, SQLite, SvelteKit, Svelte 5, Svelte Flow, TypeScript, ffmpeg, NVIDIA Nemotron, Nebius Token Factory, Nebius AI Cloud, Tavily, Docker.

## Links
- Demo: [demo URL]
- Code: https://github.com/spark-456/interlock
- Video: [YouTube link]
