// Helpers shared by the preview renderers.

import type { FileRef, Item } from '$lib/api/types';

/** Files of a kind (`image/`, `video/`, `audio/`) across items. */
export function filesOf(items: Item[], mimePrefix: string): FileRef[] {
  return items.flatMap((i) => Object.values(i.files ?? {}).filter((f): f is FileRef => !!f && f.mime.startsWith(mimePrefix)));
}

/** Where the browser loads a file from. */
export const fileUrl = (f: FileRef) => f.previewUrl ?? `/api/files/${f.id}`;

/** The item JSON as a plain object, or null. */
export const objectOf = (i: Item | undefined) =>
  i?.json && typeof i.json === 'object' && !Array.isArray(i.json) ? (i.json as Record<string, unknown>) : null;
