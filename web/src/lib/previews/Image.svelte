<script lang="ts" module>
  import type { Item } from '$lib/api/types';
  import { fileUrl, filesOf } from './files';

  export const label = 'Images';
  export const rank = 30;
  export const match = (items: Item[]) => filesOf(items, 'image/').length > 0;
</script>

<script lang="ts">
  let { items }: { items: Item[] } = $props();
</script>

<div class="grid">
  {#each filesOf(items, 'image/') as f (f.id)}<img src={fileUrl(f)} alt={f.name} loading="lazy" />{/each}
</div>

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 8px; }
  img { width: 100%; border-radius: 8px; border: 1px solid var(--border); background: var(--surface-2); }
</style>
