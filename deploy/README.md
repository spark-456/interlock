# Interlock deployment

The image builds the Svelte app, embeds it in the Rust server, and runs as UID 10001 with ffmpeg. It listens on port 8080. `/data` contains the SQLite database and uploaded files; keep it on a persistent volume. The image includes `fixtures/` (templates, recorded runs, sample files, recorded help-center pages and the sample OpenAPI spec) so a new demo session works without external providers.

## Build and verify locally

From the repository root, with Docker running:

```text
docker build -t interlock:local .
docker run --name interlock-local --rm -p 127.0.0.1:8080:8080 -v interlock-local-data:/data -e IL_PROVIDERS=fake interlock:local
```

In another terminal, run `node deploy/smoke.mjs http://127.0.0.1:8080`. It checks health, the node catalog, a fresh demo session and one seeded template's successful recorded run. It never invokes a paid provider.

## Nebius VM

Use a Nebius VM with a public IP and an attached persistent disk. Mount the disk at `/mnt/interlock-data` and make it writable by UID 10001. Install Docker with Compose, clone the repository, and copy `config/interlock.example.toml` to `/etc/interlock/interlock.toml`. Then change these settings in it (see `docs/SECURITY-REVIEW.md`):

```toml
[server]
public_url = "https://<your domain>"   # Secure cookies, OAuth callbacks
local_workspace = false                 # no sign-in-free workspace on a public host
trust_proxy = true                      # rate limits see the visitor behind Caddy

[caps]
global_daily_usd = 2.0                  # the example's 25.0 is for local use
demo_run_usd = 0.02
```

Demos replay recorded media and use only the `demo_models` roles.

Set `NEBIUS_API_KEY`, `IL_MASTER_KEY` (base64 of 32 random bytes), and any enabled connector or media provider keys through `/etc/interlock/secrets.env`, readable only by the VM administrator. Compose passes these as environment variables to the container. The image defaults to `IL_PROVIDERS=real` and `IL_CONFIG=/etc/interlock/interlock.toml`. Keep the configured storage path at `/data` (the example config uses `data` relative to the container's `/` working directory).

Set `INTERLOCK_DOMAIN` in the VM's shell to the public DNS name and point DNS at the VM's public IP. Run `docker compose -f deploy/compose.yaml up -d --build` from the repository root. Caddy serves HTTPS on ports 80 and 443 and proxies to Interlock on the private Compose network; it retains certificates in a volume. Make the demo page available without a sign-in wall. Do not expose port 8080 or the env file. After restart, confirm the same demo data remains on `/data`, then run `node deploy/smoke.mjs https://<public-host>` from outside the VM and check the URL in a private browser window. The smoke script reads no credentials and creates one isolated demo workspace.

Check the current [Nebius documentation](https://docs.nebius.com/) and the account's spending limits before provisioning. A serverless endpoint is an option only if it permits unauthenticated public HTTPS and a persistent `/data` mount. Use the VM route when either requirement is unavailable.

## Current release gate

Public deployment and URL verification require the owner's Nebius project access, persistent-disk budget, DNS name, and provider keys. Until supplied, the image and smoke script can be checked locally with fake providers; do not place secrets in Git or the image.
