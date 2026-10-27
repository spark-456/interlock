<script lang="ts">
  // Click-to-open popover anchored under its trigger. Closes on Escape and outside click.
  import type { Snippet } from 'svelte';

  let { open = $bindable(false), trigger, children, align = 'start' }: {
    open?: boolean; trigger: Snippet<[{ toggle: () => void; open: boolean }]>; children: Snippet; align?: 'start' | 'end';
  } = $props();
  let root: HTMLDivElement | undefined = $state();

  function onDoc(e: PointerEvent) {
    if (open && root && !root.contains(e.target as Node)) open = false;
  }
</script>

<svelte:document onpointerdown={onDoc} onkeydown={(e) => e.key === 'Escape' && (open = false)} />

<div class="pop" bind:this={root}>
  {@render trigger({ toggle: () => (open = !open), open })}
  {#if open}<div class="panel {align}" role="dialog">{@render children()}</div>{/if}
</div>

<style>
  .pop { position: relative; display: inline-flex; }
  .panel {
    position: absolute; top: calc(100% + 6px); z-index: 30; min-width: 200px; padding: var(--s-2);
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-lg);
  }
  .start { left: 0; }
  .end { right: 0; }
</style>
