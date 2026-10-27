<script lang="ts">
  // Shows items with the best matching renderer; other matching views (always JSON) are a click away.
  import type { Item } from '$lib/api/types';
  import SegmentedControl from '$lib/design/SegmentedControl.svelte';
  import { previewsFor } from '$lib/previews';

  let { items, total }: { items: Item[]; total?: number } = $props();
  const options = $derived(previewsFor(items));
  let chosen = $state('');
  const current = $derived(options.find((p) => p.label === chosen) ?? options[0]);
  const count = $derived(total ?? items.length);
</script>

{#if items.length === 0}
  <p class="empty">Nothing yet. Run the flow or this step to see data here.</p>
{:else if current}
  <div class="preview">
    <div class="bar">
      <span class="count">{count} {count === 1 ? 'item' : 'items'}{count > items.length ? `, first ${items.length} shown` : ''}</span>
      {#if options.length > 1}
        <SegmentedControl label="View as" options={options.map((p) => ({ value: p.label, label: p.label }))} bind:value={() => current.label, (v) => (chosen = v)} />
      {/if}
    </div>
    <current.default {items} />
  </div>
{/if}

<style>
  .preview { display: flex; flex-direction: column; gap: 10px; }
  .bar { display: flex; align-items: center; justify-content: space-between; gap: 8px; flex-wrap: wrap; }
  .count { font-size: 12px; color: var(--text-3); }
  .empty { font-size: 13px; color: var(--text-3); padding: 8px 0; }
</style>
