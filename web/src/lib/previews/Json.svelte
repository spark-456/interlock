<script lang="ts" module>
  export const label = 'JSON';
  export const rank = 0;
  export const match = () => true;
</script>

<script lang="ts">
  import type { Item } from '$lib/api/types';

  let { items }: { items: Item[] } = $props();
</script>

<pre>{JSON.stringify(items.length === 1 ? items[0] : items, null, 2)}</pre>

<style>
  pre {
    margin: 0; padding: 12px; max-height: 360px; overflow: auto; border-radius: var(--r-card); background: var(--surface-2);
    font: 12px/18px var(--font-mono); color: var(--text-2);
  }
</style>
