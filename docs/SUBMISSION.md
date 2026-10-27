# Interlock submission draft

Track: **Best Apps and Agents**. Secondary consideration: **Best Use of Tavily**.

## Devpost copy

### What we built

Interlock builds an editable backend workflow from a plain-language goal. You can start with a seeded template, see every node and connection on a canvas, test a run, inspect its events and publish the flow. It combines AI steps, branching, knowledge, media, approvals and integrations behind a modular Rust server and Svelte interface.

The generated flow remains yours to inspect and edit. That matters when an automation must be understandable before it is allowed to send a message, call an external service or spend model tokens.

Interlock builds on ideas from our earlier project Tilo, but has a new codebase, design and feature set.

### How we built it

Rust crates separate shared contracts and traits from the engine, architect, storage, knowledge, providers and nodes. Node manifests generate the catalog and settings schema. Nebius Token Factory serves the NVIDIA Nemotron models through an OpenAI-compatible adapter. Ultra is configured for the architect, Super for agents and writing, and Nano for fast classification, extraction and routing. Tavily's crawl API supplies website content to the knowledge ingestion pipeline. A Svelte canvas displays and edits flows; server-sent events stream build and run progress.

Offline tests use fake providers, local HTTP mocks and recorded outputs. A container deployment on Nebius with persistent storage and HTTPS is prepared; the public endpoint and live integrations must be verified before submission.

### Feedback on the tools

- **Nebius Token Factory:** One compatible model interface makes it straightforward to route several roles through a single gateway. We would value clearer discovery of exact model IDs and structured-output capabilities so configuration can be checked before a build starts.
- **Nebius AI Cloud:** Container hosting is a practical fit for a single server with a persistent SQLite volume. We would value an especially clear, low-friction path for a public HTTPS demo with persistent storage and externally managed secrets.
- **NVIDIA Nemotron:** The Ultra/Super/Nano range lets the app reserve heavier reasoning for flow construction while using faster models for ordinary nodes. We are testing how reliably each model follows strict JSON schemas; final performance observations require live runs.

### Challenges, results and next steps

The hard part was keeping generated flows, node settings and runtime execution aligned while preserving module boundaries and offline testability. The manifest-driven catalog and `il-sdk` testkit let modules evolve independently. The codebase includes seeded templates, a visual editor, API and MCP publishing paths, and a recorded demo mode. Before submission we still need the public Nebius URL, live Token Factory and Tavily verification, runtime screenshots, and the license change required by the hackathon checklist.

**Links to fill after verification:** public demo URL; public video URL; GitHub repo URL; screenshots.

## Demo video script (target: 2 min 40 s)

| Time | Picture | Narration |
| --- | --- | --- |
| 0:00–0:15 | Landing page and a one-sentence goal. | “Interlock turns a request into a backend workflow you can inspect and change.” |
| 0:15–0:35 | Start a build; show progress events. | “The architect chooses nodes and connects them into a working flow.” |
| 0:35–1:00 | Canvas: inspect settings, branch, knowledge and output. | “Every step stays visible. You can edit settings and logic before a run.” |
| 1:00–1:25 | Run the flow; show streamed events and output preview. | “Test a flow and follow each node as it executes.” |
| 1:25–1:45 | Show website knowledge source and Tavily-backed ingestion. | “Tavily brings website content into the flow’s knowledge base.” |
| 1:45–2:05 | Show model configuration and relevant flow steps. | “Nemotron Ultra builds; Super powers agents and writing; Nano handles fast tasks, all through Nebius Token Factory.” |
| 2:05–2:25 | Publish screen: automation, API, MCP or widget. | “Publish the same flow where your users and tools need it.” |
| 2:25–2:40 | Demo URL and project name. | “Try the public demo and explore the project code.” |

Record the actual running app and verified API calls. Replace any scene that is unavailable in the public deployment; do not present a fixture as a live provider result.
