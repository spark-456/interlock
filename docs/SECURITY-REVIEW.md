# Security review before the public deploy (Q03, 29 Oct 2026)

What was checked on `main`, what was fixed, and what the deploy must set. Findings from the capability audit (`docs/AUDIT.md`) were fixed in Q10 (workspace isolation), C07 (token refresh, send dedupe, tool previews) and B10 (demo caps).

## Checks

| Area | Result |
|---|---|
| Outbound HTTP (SSRF) | Every outbound call goes through `il_sdk::SafeHttp`: model providers, Tavily, connectors, the HTTP and OpenAPI steps, agent tools. The only `reqwest::Client::new()` in the tree is inside the Rust example of the generated integration guide. |
| Secrets in logs | No `tracing` call logs a token, key, password or authorization header; errors are logged as `error = %e`. |
| Secrets in API responses | Connections return id, app, label, status and account only (`ConnectionSummary`); API keys are shown once, at creation. |
| Cookies | Session and OAuth-state cookies are `HttpOnly; SameSite=Lax`, and `Secure` when `server.public_url` is `https://`. Lax cookies aren't sent on cross-site POSTs, so form-based CSRF against the JSON and upload endpoints doesn't carry a session. |
| CORS | Same-origin by default; only `server.cors_origins` may call with credentials. Widget chat (`POST /w/:slug/chat`) accepts any origin, which it needs to be embeddable; it serves the published flow and doesn't read the session. |
| Rate limits | `/hooks`, `/v1`, `/mcp`, widget chat and GitHub sign-in share the per-IP public limit; demo sessions have their own hourly limit. **Fixed here:** behind a proxy every request came from the proxy's address, so all visitors shared one bucket. `server.trust_proxy = true` makes the limits use the address the proxy appends to `X-Forwarded-For` (its last entry, which the client can't forge). |
| Workspace isolation | Files, builds, flows, runs and approvals check the caller's workspace (Q10). `server.local_workspace = false` turns off the sign-in-free local workspace. |
| Demo safety | Demo runs are capped at `caps.demo_run_usd`, use only `caps.demo_models`, replay recorded media, can't add connections, and builds stop at the daily caps (B10). |

## Public deploy checklist

```toml
[server]
public_url = "https://<your domain>"   # Secure cookies, OAuth callbacks
local_workspace = false                 # no sign-in-free workspace
trust_proxy = true                      # behind Caddy/nginx

[caps]
global_daily_usd = 2.0                  # keep model spend inside the free credit
demo_run_usd = 0.02
```

Environment: `IL_MASTER_KEY` (32 random bytes, base64) and the model keys. Never put secrets in `interlock.toml`.

## Left open (low risk)

- Step outputs and fixture files have no workspace owner, so anyone with a file id can read them. Ids are unguessable ULIDs, and template media is public demo content.
- A Google request that gets a 401 isn't retried after a refresh; tokens are refreshed a minute before they expire, so this only happens if Google revokes a token early.
