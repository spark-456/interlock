<script lang="ts">
  // Build log rows: elapsed time + text; the newest line is highlighted.
  import { clock } from '$lib/design/format';

  let { lines, limit }: { lines: { t: number; text: string }[]; limit?: number } = $props();
  const shown = $derived(limit ? lines.slice(-limit) : lines);
</script>

<ol class="log" aria-live="polite" aria-label="Build log">
  {#each shown as l, i (l.t + l.text)}
    <li class:latest={i === shown.length - 1}><span class="t">{clock(l.t)}</span><span class="x">{l.text}</span></li>
  {/each}
</ol>

<style>
  .log { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; justify-content: flex-end; gap: 6px; }
  li { display: flex; gap: 10px; font-size: 13px; line-height: 19px; animation: in var(--dur-node-enter) ease-out; }
  .t { flex: 0 0 34px; font: 400 12px/19px var(--font-mono); color: var(--text-4); }
  .x { color: var(--control-text); }
  .latest .x { color: var(--primary); font-weight: 600; }
  @keyframes in { from { opacity: 0; transform: translateY(4px); } }
</style>
