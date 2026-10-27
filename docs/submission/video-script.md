# Demo video script (draft, under 3 minutes)

Draft for the owner to record. Times are targets. Use the live demo URL; no sign-in. Music: none, or a track you have the rights to. Show only our own logos and the partners' wordmarks as allowed.

| Time | Screen | Say |
|---|---|---|
| 0:00–0:12 | Landing, the hero canvas building Larkspur Support | "Most teams that want an AI assistant or an automation get stuck wiring models, tools and tests. Interlock builds the whole backend from one sentence." |
| 0:12–0:30 | Click **Try the demo** → demo picker → **Build your own** | "No sign-in. I'll describe what I want, drop in our refund policy and plans, and add our admin API." Type the F-C goal: *"A support chat agent that answers from these documents and can look up invoices through our API."* Drop `refund-policy-2026.md`, `plans-and-limits.md`; add the Larkspur OpenAPI link. Attach `support-voice-memo.mp3` as **Instructions**. |
| 0:30–1:05 | Building screen | "NVIDIA Nemotron Ultra on Nebius Token Factory plans the flow. Watch it assemble: knowledge base, a router, two specialist agents, safety rules, outputs. The voice memo became instructions. Then it tests itself: when a test fails, it fixes the prompt and re-runs." Point at the build log, the stage stepper and a "fixed" card. |
| 1:05–1:30 | Editor (Open editor) | "Everything is editable. Each step has a settings form generated from its schema. I can ask for changes in plain words." Use **Ask Interlock**: *"Hand refunds over $100 to a person."* Show the diff, accept it. |
| 1:30–1:55 | Playground | "Let's talk to it." Ask *"Can I get a refund for last month's invoice?"* Show the activity chips (routed to billing, tools called, passages searched), the answer, and open the **Refund policy 2026** source: "It quotes the 2026 policy, not the old 14-day help page." |
| 1:55–2:20 | Connect page | "It's already an API. OpenAI-compatible, so existing SDKs work by changing the base URL. There's an MCP server for Claude Code or Cursor, a one-line chat widget, and an integration guide your coding agent can read." Create a key (shown once), switch the code tab to Python. |
| 2:20–2:45 | Flows → Weekly product promos → recorded run | "It's not just chat. This automation makes promo videos for new Shopify products, cuts them for every platform, and waits for approval before posting." Show the run: clips gallery, the approval card, steps with timings and cost. |
| 2:45–2:58 | Landing CTA | "Interlock: your AI backend, built from a sentence. Try the demo; the code is on GitHub." |

## Shot checklist
- Start from a fresh demo session so templates and the recorded runs are seeded.
- Before recording, check the model ids against `GET /v1/models` (the server logs them at startup) and that `NEBIUS_API_KEY` and `TAVILY_API_KEY` are set.
- Keep the browser at 1440 × 900 and zoom at 100 %.
