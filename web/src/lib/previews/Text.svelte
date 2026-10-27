<script lang="ts" module>
  import type { Item } from '$lib/api/types';
  import { objectOf } from './files';

  export const label = 'Text';
  export const rank = 5;
  const KEYS = ['text', 'post', 'answer', 'content', 'message', 'summary', 'body'];
  /** The main text of an item: a known text field, or a plain string item. */
  export const textOf = (i: Item): string | null => {
    if (typeof i.json === 'string') return i.json;
    const j = objectOf(i);
    const key = KEYS.find((k) => typeof j?.[k] === 'string');
    return key ? (j![key] as string) : null;
  };
  export const match = (items: Item[]) => items.length > 0 && items.every((i) => textOf(i) !== null);
</script>

<script lang="ts">
  let { items }: { items: Item[] } = $props();
</script>

<div class="list">
  {#each items as item, i (i)}<p>{textOf(item)}</p>{/each}
</div>

<style>
  .list { display: flex; flex-direction: column; gap: 8px; }
  p { margin: 0; padding: 10px 12px; border-radius: var(--r-card); background: var(--surface-2); font-size: 13px; line-height: 19px; white-space: pre-wrap; }
</style>
