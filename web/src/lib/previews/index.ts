// Output preview registry (ED-5). Each `*.svelte` here is a renderer that exports `match`, `rank`
// and `label` from its module script; adding a preview is adding a file.

import type { Component } from 'svelte';
import type { Item } from '$lib/api/types';

export interface Preview {
  default: Component<{ items: Item[] }>;
  /** True when this renderer can show these items. */
  match: (items: Item[]) => boolean;
  /** Higher wins when several match. */
  rank: number;
  label: string;
}

const modules = import.meta.glob<Preview>('./*.svelte', { eager: true });
const all = Object.values(modules).sort((a, b) => b.rank - a.rank);

/** Renderers that can show `items`, best first (JSON always matches). */
export function previewsFor(items: Item[]): Preview[] {
  return all.filter((p) => p.match(items));
}
