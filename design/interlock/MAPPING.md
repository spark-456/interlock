# Design → app mapping

This maps the v3 design (`design/interlock/`) onto the PRD and the workboard. **The PRD wins on architecture, routes, data, API and events.** The design wins on visuals: tokens, layout, the raised "pop" cards, copy tone, the logo and logo slots.

Web tasks must match this folder and this file. When a screen changes, update its row here.

## Screens

| ID | Design screen (design route) | PRD route | `web/src` path | Task | Differences and how they were resolved |
|---|---|---|---|---|---|
| S1 | Landing (`/`) | `/` | `routes/+page.svelte` | W07 | The hero canvas is the read-only `lib/canvas/NodeCanvas` playing the recorded Larkspur build. "Try the demo" goes to `/demo` (WS-2). |
| S2 | New backend (`/new`) | `/new` | `routes/new/+page.svelte` | W04 | The design only describes AI apps ("backend"). The PRD prompt box also creates automations, so copy is "New flow" where the kind is unknown and "backend" only for AI apps. The three steps map to AR-1: goal, attachments (Knowledge/Instructions toggle), APIs. |
| S3 | Building (`/build/:id`) | `/new/[id]/building` | `routes/new/[id]/building/` | W04 (**built in this PR**) | The route follows the PRD. Everything on screen is driven by build SSE events (PRD §8.4). Additive events (see below) carry each node's stats and the running cost. The "Inspect" button is a placeholder until the inspector (W03) exists. |
| S4 | Ready / Connect (`/b/:id/connect`) | `/flows/[id]/publish` | `routes/flows/[id]/publish/` | W06 | The PRD names it Publish. Automations show the Activate state instead of the API, MCP and widget cards (W06 card). |
| S5 | Playground (`/b/:id/playground`) | `/flows/[id]/chat` | `routes/flows/[id]/chat/` | W06 | The global "Playground" nav item has no PRD route. It opens the most recent AI app's `/flows/[id]/chat`, or `/flows` when there is none. |
| S6 | Backend overview (`/b/:id`) | `/flows/[id]` | `routes/flows/[id]/+page.svelte` | W02 (**built**), W03 (**inspector built**) | The overview is the editor canvas, with a step palette, undo/redo, tidy, save and a test run that animates the canvas from the run stream. The inspector is the node panel from PRD ED-3 (Settings / Input / Output) styled like the design's inspector: selecting a step opens it (full screen on mobile) and replaces the floating selection bar. "Run this step" needs a saved flow and only updates the panel, not the canvas's run status. The search tester and tests panels under the canvas belong to the knowledge and AI-app tasks. |
| S7 | Your backends (`/backends`) | `/flows` | `routes/flows/+page.svelte` | W05 | Lists automations and AI apps together with a kind badge. |
| S8 | Demo picker (`/demo`) | `/demo` | `routes/demo/` | W07 | Creates a demo session (`POST /api/demo/session`) before opening a template. |
| M1 | Landing (mobile) | `/` @390 | same as S1 | W07 | Responsive layout of S1. |
| M2 | Building (mobile) | `/new/[id]/building` @390 | same as S3 (`BuildList.svelte`) | W04 (**built in this PR**) | Below 768 px the canvas becomes the grouped list. The same list is the canvas "List view" for accessibility (ED-1). |
| M3 | Connect (mobile) | `/flows/[id]/publish` @390 | same as S4 | W06 | Responsive layout of S4. |
| M4 | Playground (mobile) | `/flows/[id]/chat` @390 | same as S5 | W06 | Responsive layout of S5. |
| — | NodeCanvas component | used by `/`, `/new/[id]/building`, `/flows/[id]`, run view | `lib/canvas/` | W02 (foundation **in this PR**) | Built on Svelte Flow (`@xyflow/svelte`) for drag, pan and zoom, as the PRD requires. The design describes its own pointer handling; Svelte Flow provides the same behaviour. |
| SP1 | Spec sheet | `/dev/components` (gallery) | `lib/design/` | W01 | Base components are **in this PR**. The gallery page is still open on W01. |
| SP2 | Brand sheet | — | `lib/design/Logo.svelte`, `static/favicon.svg` | W01 | The mark, lockup and app icon come from `brand/*.svg`. |

## App shell

| Design | Resolution |
|---|---|
| Sidebar items: New backend, Backends, Playground, Logs, Docs, Settings | **PRD routes win.** Items are New (→ `/new`), Flows (→ `/flows`), Templates, Approvals, Connections and Settings (PRD §7.3). Logs live in each flow's runs, and Docs is the per-flow integration guide (PB-5), so neither gets a top-level item. Styling follows the design exactly. |
| Sidebar collapses to 64 px on canvas screens (S3, S6) | The `collapsed` prop on `Sidebar` is set by `/new/[id]/building` and `/flows/[id]`. |
| Mobile tab bar: Backends, New, Playground, Settings | Flows, New, Approvals, Settings. Approvals replaces Playground because the PRD makes the approvals inbox mobile-first (WS-4 P0 mobile). |
| Account block "Dana Ruiz · Larkspur · Free" | Comes from `GET /api/me`. In demo mode it shows "Demo workspace". |

## Node categories

The design colours nodes by their role in an AI app (source, kb, router, agent, tool, guard, tests, output). The PRD's node catalog uses `Category` (trigger, ai, knowledge, media, logic, data, app, human, output). **PRD data wins.** `lib/canvas/categories.ts` derives each node's tile from its category, refined by type id:

| PRD category / type | Design tile |
|---|---|
| `knowledge.*` | kb (violet) |
| `ai.agent` | agent (teal) |
| `ai.decide`, `ai.classify`, `logic.switch`, `logic.if` | router (sky) |
| other `ai.*` | agent (teal) |
| `data.openapi`, `data.http` | tool (amber) |
| other `data.*`, `logic.*` | source (blue) / logic grey |
| `trigger.*` | source (blue); webhook/schedule icons |
| `human.*` | guard (rose) |
| `output.*` | output (indigo) |
| `media.*` | media (pink, from the T00 token set) |
| `app.*` | white tile + 1 px border with the app's logo slot |
| build-only test suite (`build.tests`) | tests (green) |

## Events (PRD §8.4) and the additive extensions this PR adds

These are additive under AGENTS.md §6: new optional fields and new event types that the UI may ignore. No existing event changed meaning.

- `build.flow_patch` adds nodes, edges and groups. Group labels become the canvas column labels (Inputs, Understanding, Agents, Safety, Outputs).
- `build.stage` drives the 7-step stepper: `read`, `index`, `team`, `prompts`, `tools`, `test`, `live`.
- `build.log` feeds the build log. Times are elapsed from the first event.
- `build.test_result` and `build.fix` drive the test-suite node and the "Moments later" card. **Added:** optional `title`, `before`, `after` and `source` on `build.fix`, for the before→after chips.
- **New** `build.node` {status, subtitle?, label?, summary?, stats?, progress?, icon?, tone?} (with `nodeId`): a node's state, type label, pill text, list-view summary, stat rows and optional icon/tone overrides. Statuses: waiting, building, ready, fixed, warning, failed.
- **New** `build.cost` {costUsd}: the running "Cost so far".

## Places where the PRD was followed over the design

1. Routes: `/new/[id]/building`, `/flows`, `/flows/[id]`, `/flows/[id]/publish` and `/flows/[id]/chat` instead of `/build/:id`, `/backends` and `/b/:id/...`.
2. Navigation items and mobile tabs (see App shell).
3. "Flow" is the data-model term. "Backend" appears only in copy about AI apps.
4. Node colours come from catalog categories, not the design's fixed roles (see Node categories).
5. The canvas uses Svelte Flow, as PRD §7.3 requires. The design's own pointer and zoom code was not ported.
6. Every Building state comes from build events (PRD §8.4). The design's static stats, timer and cost are now event data via additive events.
7. The design's "Outputs" (API, MCP server, chat widget) are publish targets in the PRD (PB-1/2/4). In the recorded build they appear as `output.respond` nodes, one per channel. F-C's chat trigger is left out of the recorded Building fixture to match the design's five columns. The F-C template (R01) includes it.
8. "Safety rules" (guardrails, AR-9) has no P0 catalog node yet. The fixture uses `ai.decide` named "Safety rules", with `tone: guard` and `icon: shield` from `build.node`. Task N01 (`ai.guardrails` node) was added to the workboard.
9. The test suite (AR-9) is a build artifact, not a flow node. During a build it appears as a build-only node of type `build.tests` (column Agents), linked to the Billing agent by a dotted vertical edge (`from.port = "bottom"`). The saved flow (`flw_larkspur`) does not contain it. A01/A02 must emit it the same way.
10. Inputs (website, documents, API spec) are flow nodes (`knowledge.add`, `data.parse_document`, `data.openapi` with `operationId: "*"`), shown with the design's "source" tone through `build.node` overrides.
11. The design's "Refine" button opens the PRD's Ask Interlock bar (ED-9, AR-8): a change request returns a JSON-Patch proposal, drawn on the canvas as added (green), changed (amber) and removed (faded red) steps, with Accept and Discard. Editing and saving wait until the proposal is settled.
