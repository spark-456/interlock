# Interlock design — use it in Claude Code

1. Unzip this folder into your project, e.g. `your-repo/design/interlock/`.
2. Open Claude Code in the repo and paste the prompt below (edit the stack line if yours differs).
3. Build one screen at a time, starting with the NodeCanvas and S3 Building — that's the hero of the demo video.

## Starter prompt for Claude Code

```
Read design/interlock/DESIGN.md first, then the screenshots in design/interlock/screens/
and the matching markup in design/interlock/source/ (design-tool files: port them, don't ship them).

Build the Interlock web UI in Svelte (served by our Rust binary; build progress arrives over SSE).
1. Copy design/interlock/tokens/tokens.css into the app and use the CSS variables everywhere.
2. Build the shared components listed in DESIGN.md, then the NodeCanvas exactly as specified:
   draggable nodes, pan, zoom, fit, minimap, bezier edges, live states driven by SSE events.
3. Then build the screens in this order: S3 Building, S4 Connect, S2 New backend, S6 Overview,
   S5 Playground, S7 Backends, S8 Demo, S1 Landing, then the mobile layouts (M1–M4).
Match the screenshots pixel-close: spacing, radii, the "pop" card shadows, copy and sample data.
After each screen, run the app, take a screenshot and compare it with the PNG before moving on.
```

## What's inside

- `DESIGN.md` — the handoff spec (tokens, shell, NodeCanvas behaviour and geometry, components, rules)
- `tokens/` — `tokens.css` and `tokens.json`
- `screens/` — PNG renders of all 15 boards
- `source/` — the exact markup behind each board
- `brand/` — logo mark, app icon, horizontal lockup (SVG)
