<script lang="ts">
  // Mobile bottom sheet (inspector, node details, menus).
  import type { Snippet } from 'svelte';

  let { open = $bindable(false), title, children }: { open?: boolean; title: string; children: Snippet } = $props();
</script>

<svelte:window onkeydown={(e) => open && e.key === 'Escape' && (open = false)} />

{#if open}
  <div class="scrim" role="presentation" onclick={() => (open = false)}></div>
  <div class="sheet" role="dialog" aria-modal="true" aria-label={title}>
    <span class="grab" aria-hidden="true"></span>
    <h2>{title}</h2>
    {@render children()}
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; background: var(--scrim); z-index: 40; }
  .sheet {
    position: fixed; left: 0; right: 0; bottom: 0; z-index: 41; max-height: 85vh; overflow: auto;
    padding: var(--s-2) var(--s-4) var(--s-6); background: var(--surface); border-radius: var(--r-panel) var(--r-panel) 0 0;
    box-shadow: var(--shadow-lg); animation: up var(--dur-panel) var(--ease);
  }
  .grab { display: block; width: 40px; height: 4px; border-radius: 2px; background: var(--border-strong); margin: 0 auto var(--s-3); }
  h2 { font: var(--t-h3); font-weight: 700; margin-bottom: var(--s-3); }
  @keyframes up { from { transform: translateY(24px); opacity: 0; } }
</style>
