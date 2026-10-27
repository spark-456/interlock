<script lang="ts">
  // "Moments later" card: the latest automatic fix, with before → after chips and the source.
  import type { Fix } from '$lib/canvas/buildState';
  import Chip from '$lib/design/Chip.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';

  let { fixes, fixing, failing = 0 }: { fixes: Fix[]; fixing: boolean; failing?: number } = $props();
  const latest = $derived(fixes[fixes.length - 1]);
</script>

{#if latest}
  <section class="fix" aria-live="polite" data-testid="fix-card">
    <header>
      <span class="kicker">Moments later</span>
      {#if fixing}<StatusPill tone="building" label="Fixing {Math.max(failing, fixes.length)}…" />
      {:else}<StatusPill tone="fixed" label="Fixed {fixes.length} {fixes.length === 1 ? 'issue' : 'issues'}" />{/if}
    </header>
    <div class="body">
      <span class="tile"><Icon name="tests" size={18} color="var(--cat-tests-icon)" /></span>
      <div class="text">
        <div class="title">{latest.title ?? latest.description}</div>
        {#if latest.title}<div class="desc">{latest.description}</div>{/if}
        {#if latest.before || latest.after || latest.source}
          <div class="chips">
            {#if latest.before}<Chip kind="diff-old">{latest.before}</Chip>{/if}
            {#if latest.before && latest.after}<Icon name="arrow-right" size={14} color="var(--text-3)" />{/if}
            {#if latest.after}<Chip kind="diff-new">{latest.after}</Chip>{/if}
            {#if latest.source}<Chip kind="source">{latest.source}</Chip>{/if}
          </div>
        {/if}
      </div>
    </div>
  </section>
{/if}

<style>
  .fix {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-panel);
    box-shadow: var(--shadow-sheet); overflow: hidden; animation: up var(--dur-panel) var(--ease);
  }
  header { display: flex; align-items: center; gap: 8px; padding: 10px 14px; background: var(--surface-3); border-bottom: 1px solid var(--border); }
  .kicker { font-size: 12px; font-weight: 700; letter-spacing: .06em; text-transform: uppercase; color: var(--text-3); margin-right: auto; }
  .body { padding: 14px; display: flex; gap: 12px; }
  .tile {
    flex: 0 0 32px; width: 32px; height: 32px; border-radius: 9px; display: flex; align-items: center; justify-content: center;
    background: var(--cat-tests-tile); border: 1px solid var(--cat-tests-edge);
  }
  .text { font-size: 13px; line-height: 20px; min-width: 0; }
  .title { font-weight: 700; }
  .desc { color: var(--text-2); }
  .chips { display: flex; flex-wrap: wrap; align-items: center; gap: 6px; margin-top: 8px; }
  @keyframes up { from { opacity: 0; transform: translateY(12px); } }
</style>
