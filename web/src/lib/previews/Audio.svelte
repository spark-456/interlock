<script lang="ts" module>
  import type { Item } from '$lib/api/types';
  import { fileUrl, filesOf } from './files';

  export const label = 'Audio';
  export const rank = 30;
  export const match = (items: Item[]) => filesOf(items, 'audio/').length > 0;
</script>

<script lang="ts">
  let { items }: { items: Item[] } = $props();
</script>

<div class="list">
  {#each filesOf(items, 'audio/') as f (f.id)}
    <figure>
      <audio src={fileUrl(f)} controls preload="metadata"></audio>
      <figcaption>{f.name}</figcaption>
    </figure>
  {/each}
</div>

<style>
  .list { display: flex; flex-direction: column; gap: 10px; }
  figure { margin: 0; display: flex; flex-direction: column; gap: 4px; }
  audio { width: 100%; }
  figcaption { font: 12px var(--font-mono); color: var(--text-3); }
</style>
