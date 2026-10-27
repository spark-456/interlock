# Interlock — design handoff (v3, P0)

This folder is the design from the Claude Design canvas, packaged for building. It is the **spec**, not production code:

- `screens/*.png` — what each screen must look like (rendered from the canvas).
- `source/*.dc.html` — the exact markup and inline styles behind each screen. Read these for precise spacing, colours, copy and sample data. They use a design-tool format (`<x-dc>`, `{{holes}}`, `<sc-for>`, `<sc-if>`, a `class Component extends DCLogic` script). **Do not ship them; port them** to real components.
- `tokens/tokens.css` and `tokens/tokens.json` — the design tokens. Use them as-is.
- `brand/*.svg` — the logo mark, app icon and lockup.

Target stack (from the product brief): a lightweight **Svelte** web app served by a single **Rust** binary; builds stream over **Server-Sent Events**; canvas = **DOM node cards + SVG edges** (no WebGL, no canvas libraries required). Fonts: Plus Jakarta Sans + JetBrains Mono, self-hosted.

## Screens

| ID | Screen | Route | Source | Image |
|---|---|---|---|---|
| S1 | Landing | `/` | `source/Landing.dc.html` | `screens/Landing.png` |
| S2 | New backend | `/new` | `source/NewBackend.dc.html` | `screens/NewBackend.png` |
| S3 | Building (hero moment) | `/build/:id` | `source/Building.dc.html` | `screens/Building.png` |
| S4 | Ready / Connect | `/b/:id/connect` | `source/Connect.dc.html` | `screens/Connect.png` |
| S5 | Playground | `/b/:id/playground` | `source/Playground.dc.html` | `screens/Playground.png` |
| S6 | Backend overview | `/b/:id` | `source/Overview.dc.html` | `screens/Overview.png` |
| S7 | Your backends | `/backends` | `source/Backends.dc.html` | `screens/Backends.png` |
| S8 | Demo picker | `/demo` | `source/Demo.dc.html` | `screens/Demo.png` |
| M1 | Landing (mobile) | `/` @390 | `source/MLanding.dc.html` | `screens/MLanding.png` |
| M2 | Building (mobile) | `/build/:id` @390 | `source/MBuilding.dc.html` | `screens/MBuilding.png` |
| M3 | Connect (mobile) | `/b/:id/connect` @390 | `source/MConnect.dc.html` | `screens/MConnect.png` |
| M4 | Playground (mobile) | `/b/:id/playground` @390 | `source/MPlayground.dc.html` | `screens/MPlayground.png` |
| SP1 | Spec sheet | — | `source/SpecV3.dc.html` | `screens/SpecV3.png` |
| SP2 | Brand sheet | — | `source/BrandV3.dc.html` | `screens/BrandV3.png` |
| — | NodeCanvas component | used by S1, S3, S6 | `source/BackendCanvas.dc.html` | `screens/BackendCanvas.png` |

Desktop screens are designed at 1440 wide; mobile at 390. Breakpoints: 1440, 1280, 1024, 768, 390. Below 768 the canvas becomes the vertical grouped list (M2).

## App shell

- **Sidebar** 240 px (collapses to 64 px on canvas screens S3/S6): logo; **New backend** (primary); Backends, Playground, Logs, Docs, Settings; at the bottom a "Built with" strip (three 20 px partner slots) and the account.
- Active nav item: `--primary-soft` background, `--primary` text and icon.
- **Top bar** 64 px, `--surface`, bottom border `--border`.
- **Inspector / right panel** 400 px (Playground's right rail is 360 px).
- Mobile: top bar (logo or back + title + menu) and a bottom tab bar (Backends, New, Playground, Settings), 44 px+ targets.

## The "pop" look (requested: cards that stand out)

Every card and node uses `--card-gradient` + 1 px `--border` + `--shadow-pop` (a raised 3 px ledge plus a soft drop). Hover lifts 2 px with `--shadow-pop-hover`; selected adds the teal ring `--shadow-pop-selected` and lifts 3 px; dragging lifts 6 px, tilts −0.6°, and uses `--shadow-pop-drag`. Primary buttons have a 2 px darker ledge (`--shadow-button-primary`). Icon tiles: 32 px (nodes) or 36–40 px, radius 9–10, `linear-gradient(180deg, #FFF -40%, <tile> 70%)`, 1 px `<edge>` border, `inset 0 1px 0 rgba(255,255,255,.9)`.

## NodeCanvas (S3, S6, landing hero) — the key component

**Behaviour (must be built, not faked):**
- Drag a node: pointer down on a card captures the pointer; move updates the node's world x/y by `delta / zoom`; edges recompute every frame. A press without movement (< 3 px) selects the node and opens the inspector.
- Drag empty canvas to pan. Zoom with the toolbar (−, %, +, fit) and ctrl/⌘ + wheel around the cursor. Zoom range 0.35–1.5. "Fit" frames all nodes with 24 px padding.
- Minimap (180 × 120, bottom-right) shows all nodes and the viewport rectangle; toggle from the toolbar.
- Background: 20 px dot grid that scales and pans with the view (`background-size` = 20·zoom, `background-position` = pan).
- `touch-action: none` on the canvas; keyboard: nodes are focusable, arrow keys move between nodes, Enter opens the inspector. Provide a "List view" alternative (the M2 grouped list) for accessibility.

**Geometry:** node 264 px wide; height = 116 + 22 × statRows (2 rows → 160, 3 → 182, 1 → 138). Columns at x = 0, 304, 608, 912, 1216 (gap 40) with faint uppercase labels 44 px above: Inputs, Understanding, Agents, Safety, Outputs. Default positions are in `source/BackendCanvas.dc.html` (`base()`).

**Card anatomy:** padding 14; header = 32 px icon tile + title (14/700) + type label (12/500 `--text-3`); body = stat rows (12 px; label `--text-2`, value in mono) above a 1 px `#F2F4F7` divider; footer = status pill (24 px). Handles: 10 px circles centred vertically on left (in) and right (out) edges, 1.5 px `--border-strong` ring, `--primary` on hover/selected; sources have no in-handle, outputs no out-handle.

**Edges:** cubic bezier from source right-middle to target left-middle: `M sx sy C sx+d sy, tx−d ty, tx ty` with `d = max(48, |tx−sx| / 2)`. 1.5 px `#B4BCC8`, round caps. Queued: `#D0D5DD`, dash `4 6`. Active (data flowing): a `#B9DEDA` base line with a `--primary` dash `6 8` animated via `stroke-dashoffset` over 900 ms. Failed: `--error`. The test suite hangs under the Billing agent with a dotted (`2 5`) vertical bezier.

**States** (pill text in brackets): Waiting (60% opacity, dashed border, no shadow), Building (3 px primary progress bar on the card's top edge, spinner, "Building"), Ready ("Ready", green dot), Fixed ("Fixed 2 issues", primary-soft, check), Warning ("8 endpoints skipped"), Failed (red ring, "Failed · Retry"). Drive every change from real SSE build events — never a fake timeline. Node enter 220 ms (fade + 8 px rise), pill change 160 ms. Respect `prefers-reduced-motion` (no dash animation).

## Components to build

Button (primary, secondary, ghost, danger; S 32 / M 40 / L 48; loading keeps width) · IconButton · Input · TextArea · Select · SegmentedControl · Toggle · Checkbox · Chip (example, filter, source, tool) · StatusPill · Tabs · Tooltip · Popover · Modal · BottomSheet · Toast · Banner (Demo, Offline, Cap) · Sidebar · TopBar · TabBar · DropZone · FileRow (with Knowledge / Instructions toggle) · ApiCard · AppTile (logo slot) · PartnerSlot · BuiltWithStrip · NodeCard · Edge · CanvasToolbar · Minimap · BuildStepper · BuildLog · Inspector · PromptEditor · SearchTester · EndpointCard · TestList · ConnectCard · CodeTabs · CopyField (masked / reveal-once / "Copied" for 1.5 s) · ChatMessage · ActivityTrail · SourceChip · SuggestedAction · Composer · BackendCard · EmptyState · Skeleton.

Every interactive component needs default, hover, focus (2 px primary ring, 2 px offset), pressed, disabled and loading states — see `screens/SpecV3.png`.

## Logos

- Product mark: two interlocking rounded squares (`brand/interlock-mark.svg`); app icon = white mark on a teal rounded square (`brand/interlock-app-icon.svg`).
- **Partner slots** (NVIDIA, Nebius, Tavily) and **app slots** (OpenAI SDK, LangChain, Vercel AI SDK, Claude Code, Cursor, VS Code, Slack, Notion, Google Drive, GitHub, Stripe, Zendesk, HubSpot, Shopify) are placeholders. Build them as components that take an optional `src` for the official SVG and fall back to the name/first letter. Never draw third-party logos by hand.

## Content rules

Sample data is the "Larkspur Support" story used on every screen (see the source files). Sentence case, short, warm; no "magic", "revolutionary", "supercharge"; no exclamation marks. Users see "assistant", "agent", "knowledge", "tools" — "chunks", "embeddings", "tokens" only in Advanced, Knowledge and Logs. Models appear only in "Built with" and an inspector's Advanced section.

## Accessibility

WCAG 2.2 AA; visible focus everywhere; status = text + colour; 44 px touch targets on mobile; canvas has a list view; chat announces new messages politely.
