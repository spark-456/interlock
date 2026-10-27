<script lang="ts" module>
  import type { Item } from '$lib/api/types';
  import { objectOf } from './files';

  export const label = 'Table';
  export const rank = 10;
  type Row = Record<string, unknown>;
  const flat = (r: unknown): r is Row =>
    !!r && typeof r === 'object' && !Array.isArray(r) && Object.values(r).every((v) => v === null || typeof v !== 'object');

  /** Rows: several flat items, or one item holding a list of flat records. */
  export function rowsOf(items: Item[]): Row[] {
    const objs = items.map(objectOf);
    if (objs.length > 1 && objs.every(flat)) return objs as Row[];
    const list = Object.values(objs[0] ?? {}).find((v) => Array.isArray(v) && v.length > 0 && v.every(flat));
    return (list as Row[] | undefined) ?? [];
  }
  export const match = (items: Item[]) => rowsOf(items).length > 0;
</script>

<script lang="ts">
  let { items }: { items: Item[] } = $props();
  const rows = $derived(rowsOf(items));
  const cols = $derived([...new Set(rows.flatMap((r) => Object.keys(r)))]);
</script>

<div class="wrap">
  <table>
    <thead><tr>{#each cols as c (c)}<th scope="col">{c}</th>{/each}</tr></thead>
    <tbody>
      {#each rows as r, i (i)}<tr>{#each cols as c (c)}<td>{r[c] ?? ''}</td>{/each}</tr>{/each}
    </tbody>
  </table>
</div>

<style>
  .wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: var(--r-card); }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th { text-align: left; padding: 6px 10px; background: var(--surface-2); color: var(--text-3); font-weight: 600; white-space: nowrap; }
  td { padding: 6px 10px; border-top: 1px solid var(--border); font-family: var(--font-mono); white-space: nowrap; max-width: 220px; overflow: hidden; text-overflow: ellipsis; }
</style>
