// Node palette (ED-2): the catalog grouped by category and filtered by a search query.

import type { Category, NodeManifest } from '$lib/api/types';

/** Drag-and-drop payload type carrying a node type id. */
export const PALETTE_MIME = 'application/x-interlock-node';

const ORDER: [Category, string][] = [
  ['trigger', 'Triggers'], ['ai', 'AI'], ['knowledge', 'Knowledge'], ['logic', 'Logic'], ['data', 'Data'],
  ['human', 'Human'], ['media', 'Media'], ['app', 'Apps'], ['output', 'Outputs']
];

export interface PaletteGroup {
  label: string;
  items: NodeManifest[];
}

/** Catalog entries matching `query` (title, description or type id), grouped in palette order. */
export function paletteGroups(nodes: NodeManifest[], query: string): PaletteGroup[] {
  const q = query.trim().toLowerCase();
  const hits = nodes.filter((m) => !q || `${m.title} ${m.description} ${m.typeId}`.toLowerCase().includes(q));
  return ORDER.map(([cat, label]) => ({ label, items: hits.filter((m) => m.category === cat) })).filter((g) => g.items.length > 0);
}
