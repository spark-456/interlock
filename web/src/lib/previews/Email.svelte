<script lang="ts" module>
  import type { Item } from '$lib/api/types';
  import { objectOf } from './files';

  export const label = 'Email';
  export const rank = 20;
  const isEmail = (i: Item) => {
    const j = objectOf(i);
    return !!j && typeof j.subject === 'string' && (typeof j.body === 'string' || typeof j.text === 'string');
  };
  export const match = (items: Item[]) => items.length > 0 && items.every(isEmail);
</script>

<script lang="ts">
  let { items }: { items: Item[] } = $props();
  const line = (j: Record<string, unknown> | null) => [j?.from && `From ${j.from}`, j?.to && `To ${j.to}`].filter(Boolean).join(' · ');
</script>

<div class="list">
  {#each items.map(objectOf) as j, i (i)}
    <article>
      {#if line(j)}<div class="meta">{line(j)}</div>{/if}
      <h4>{j?.subject}</h4>
      <p>{j?.body ?? j?.text}</p>
    </article>
  {/each}
</div>

<style>
  .list { display: flex; flex-direction: column; gap: 10px; }
  article { padding: 12px; border: 1px solid var(--border); border-radius: var(--r-card); background: var(--surface); }
  .meta { font-size: 12px; color: var(--text-3); }
  h4 { margin: 4px 0; font-size: 14px; font-weight: 700; }
  p { margin: 0; font-size: 13px; line-height: 19px; color: var(--text-2); white-space: pre-wrap; }
</style>
