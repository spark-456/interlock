# CLAUDE.md

All project rules live in AGENTS.md so Claude Code and Codex follow the same ones.

@AGENTS.md

Claude-specific notes:
- Work autonomously through the whole task card; don't stop to ask for approval between steps.
- Prefer a separate git worktree per task so parallel sessions never share a working directory.
- When you finish a task and context remains, claim the next ready task per AGENTS.md §3.
- Keep the owner's progress tracker current (https://claude.ai/artifact/DLWEuh1TMRF7vpYPEWeD3e): after a claim, merge, PR or board change, run `cargo xtask progress` and upload the changed files from `target/progress/` to the tracker's datasets of the same names.
