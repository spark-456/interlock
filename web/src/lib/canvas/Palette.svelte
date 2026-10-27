<script lang="ts">
  // Node palette (ED-2): search the catalog; drag an item onto the canvas or an edge, or click to
  // add it next to the selection. In "replace" mode a click swaps the selected node's type (ED-6).
  import type { NodeManifest } from '$lib/api/types';
  import Icon from '$lib/design/Icon.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import { iconFor, toneFor, toneVars } from './categories';
  import { PALETTE_MIME, paletteGroups } from './palette';

  let {
    nodes,
    replacing = null,
    onpick,
    onclose
  }: {
    nodes: NodeManifest[];
    /** Title of the node being replaced, or null when adding. */
    replacing?: string | null;
    onpick: (m: NodeManifest) => void;
    onclose: () => void;
  } = $props();

  let query = $state('');
  const groups = $derived(paletteGroups(nodes, query));
</script>

<aside class="palette" aria-label={replacing ? `Replace ${replacing}` : 'Add a step'} data-testid="palette">
  <div class="head">
    <h2>{replacing ? `Replace ${replacing} with…` : 'Add a step'}</h2>
    <IconButton icon="x" label="Close" onclick={onclose} />
  </div>
  <label class="search">
    <Icon name="search" size={16} color="var(--text-3)" />
    <input type="search" placeholder="Search steps" aria-label="Search steps" bind:value={query} />
  </label>
  <div class="groups">
    {#each groups as g (g.label)}
      <section>
        <h3>{g.label}</h3>
        {#each g.items as m (m.typeId)}
          {@const v = toneVars(toneFor(m.typeId))}
          <button
            type="button"
            class="item"
            draggable={!replacing}
            ondragstart={(e) => e.dataTransfer?.setData(PALETTE_MIME, m.typeId)}
            onclick={() => onpick(m)}
            title={m.description}
            data-testid="palette-{m.typeId}"
          >
            <span class="tile" style:--tile={v.tile} style:border-color={v.edge}><Icon name={iconFor(m.typeId, m.icon)} size={16} color={v.icon} /></span>
            <span class="t">{m.title}</span>
            {#if m.status === 'stub'}<span class="soon">Soon</span>{/if}
          </button>
        {/each}
      </section>
    {:else}
      <p class="empty">No step matches “{query}”.</p>
    {/each}
  </div>
</aside>

<style>
  .palette {
    width: 280px; height: 100%; display: flex; flex-direction: column; background: var(--surface);
    border-right: 1px solid var(--border);
  }
  .head { display: flex; align-items: center; justify-content: space-between; padding: 14px 12px 8px 16px; }
  h2 { font-size: 14px; line-height: 20px; font-weight: 700; }
  .search {
    margin: 0 12px 8px; display: flex; align-items: center; gap: 8px; height: 36px; padding: 0 10px;
    border: 1px solid var(--border); border-radius: var(--r-control); background: var(--surface);
  }
  .search:focus-within { box-shadow: var(--focus-ring); }
  .search input { flex: 1; min-width: 0; border: 0; outline: 0; background: transparent; font: inherit; font-size: 14px; color: var(--text); }
  .groups { flex: 1; overflow-y: auto; padding: 0 8px 16px; }
  h3 {
    padding: 12px 8px 4px; font-size: 11px; line-height: 16px; font-weight: 700; letter-spacing: .06em;
    text-transform: uppercase; color: var(--text-4);
  }
  .item {
    width: 100%; display: flex; align-items: center; gap: 10px; padding: 6px 8px; border: 0; border-radius: var(--r-control);
    background: transparent; color: var(--text); font: inherit; font-size: 13px; text-align: left; cursor: grab;
  }
  .item:hover { background: var(--surface-2); }
  .item:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .tile {
    flex: 0 0 28px; width: 28px; height: 28px; border-radius: 8px; border: 1px solid;
    background: linear-gradient(180deg, var(--surface) -40%, var(--tile) 70%); display: flex; align-items: center; justify-content: center;
  }
  .t { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 500; }
  .soon { font-size: 11px; color: var(--text-4); }
  .empty { padding: 16px 8px; color: var(--text-3); font-size: 13px; }
</style>
