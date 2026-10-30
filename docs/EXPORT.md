# Export as code (Q36)

Someone who builds and tests a flow in Interlock can take it into their own codebase and run it on
their own machines, with no calls to the hosted service and no outside calls except the model and
API providers they configure.

## The choice: a data bundle run by the same engine

An export is a folder of plain files (a zip) that the open-source Interlock engine loads at startup:

```
support-agent/
  README.md              what it does, how to run it, every endpoint, every setting
  INTERLOCK.md           the same for coding agents, plus how to change the flow
  .env.example           placeholders for exactly the keys this flow needs
  .gitignore             .env and data/
  compose.yaml           `docker compose up`: builds the engine from source, loads this folder
  interlock.toml         model roles and providers, engine limits, spend caps (no secrets)
  interlock.bundle.json  manifest: format version, flows, files, connections and their env names
  flows/<flow id>.json   the flow definition (versioned JSON)
  prompts/<step>.<field>.md   each step's prompt or instructions, referenced from the flow
  files/<file id>/<name> documents, API specs and sample files the flow uses
```

On start, `il-server` with `IL_BUNDLE=<folder>` reads the manifest and, idempotently:

1. stores the bundle's files and points the flow at them,
2. fills each `@prompts/…` reference from its file,
3. saves the flow (a new version only when it changed),
4. saves each connection from its env vars (encrypted with `IL_MASTER_KEY`, as everywhere),
5. registers `INTERLOCK_API_KEY` as the workspace's API key and `INTERLOCK_WEBHOOK_TOKEN` as the
   webhook token,
6. turns the flow on.

Everything after that is the normal server: the same OpenAI-compatible API (`/v1`), per-app and
platform MCP (`/mcp`), widget (`/w`) and webhooks (`/hooks`), so code written against the hosted
app works unchanged once its base URL and key point at the local one.

### Why not the other shapes

- **Generated source code** (a TypeScript or Python program per flow) would reimplement 60+ step
  types, the agent loop, knowledge search, approvals, retries and spend caps for every target
  language, and would drift from what the user tested. The bundle runs the code they tested.
- **A library crate or SDK** only serves Rust codebases and still needs storage, triggers and
  HTTP endpoints around it. An HTTP service with an OpenAI-compatible API works from any language,
  and the engine crates stay available to Rust users who want to embed them later.

## Delivery

- **Publish page → Export**: downloads `<slug>-interlock.zip` (`GET /api/flows/:id/export`).
- **Coding agents**: the platform MCP tool `export_flow` returns a ready `curl … | tar` command
  for `GET /mcp/flows/:id/export` (Bearer API key), so an agent can pull the bundle into a repo.
- Pushing to a GitHub repository is left for later (WORKBOARD follow-up).

## Secrets

Nothing secret leaves the hosted service:

- connections export only their id, app, label and field names; values come from env vars named in
  `.env.example` (placeholders only);
- the hosted API keys and webhook tokens are never exported; the bundle has its own;
- `interlock.toml` copies only the model, provider, engine and cap sections, and drops any key
  whose name mentions a key, secret, token or password.

## Runtime defaults

- `local_workspace = false`: the bundle is headless, reachable only with its API key or webhook
  token. Runs and approvals are available through the platform MCP (`/mcp`). Set it to `true` to
  use the Interlock app on a trusted machine.
- compose publishes the port on `127.0.0.1` only and keeps data in a named volume.
- `IL_PROVIDERS=fake` runs the bundle fully offline (tests, CI smoke checks).
