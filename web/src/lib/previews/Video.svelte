<script lang="ts" module>
  import type { Item } from '$lib/api/types';
  import { fileUrl, filesOf } from './files';

  export const label = 'Videos';
  export const rank = 30;
  export const match = (items: Item[]) => filesOf(items, 'video/').length > 0;
</script>

<script lang="ts">
  let { items }: { items: Item[] } = $props();
</script>

<div class="list">
  {#each filesOf(items, 'video/') as f (f.id)}
    <figure>
      <video src={fileUrl(f)} controls preload="metadata"><track kind="captions" /></video>
      <figcaption>{f.name}</figcaption>
    </figure>
  {/each}
</div>

<style>
  .list { display: flex; flex-direction: column; gap: 12px; }
  figure { margin: 0; display: flex; flex-direction: column; gap: 4px; }
  video { width: 100%; border-radius: 8px; background: var(--surface-2); }
  figcaption { font: 12px var(--font-mono); color: var(--text-3); }
</style>
