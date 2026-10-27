// Logic behind the interactive design components, kept out of the .svelte files so it can be tested.

/** Menu keyboard movement: the item index after `key`, wrapping; `null` for keys the menu ignores. */
export function menuMove(index: number, key: string, count: number): number | null {
  if (count === 0) return null;
  switch (key) {
    case 'ArrowDown':
      return (index + 1) % count;
    case 'ArrowUp':
      return (index - 1 + count) % count;
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
    default:
      return null;
  }
}

/** Splits dropped or picked files into accepted ones and a reason for each rejected one. */
export function pickFiles(
  files: File[],
  { accept = [], maxBytes = Infinity }: { accept?: string[]; maxBytes?: number }
): { ok: File[]; rejected: { file: File; reason: string }[] } {
  const ok: File[] = [];
  const rejected: { file: File; reason: string }[] = [];
  const matches = (f: File) =>
    !accept.length ||
    accept.some((a) => (a.startsWith('.') ? f.name.toLowerCase().endsWith(a.toLowerCase()) : a.endsWith('/*') ? f.type.startsWith(a.slice(0, -1)) : f.type === a));
  for (const file of files) {
    if (!matches(file)) rejected.push({ file, reason: 'This file type isn’t supported.' });
    else if (file.size > maxBytes) rejected.push({ file, reason: `Files must be ${bytes(maxBytes)} or smaller.` });
    else ok.push(file);
  }
  return { ok, rejected };
}

/** "2.4 MB", "820 KB", "12 B". */
export function bytes(n: number): string {
  if (n >= 1024 * 1024) return `${+(n / (1024 * 1024)).toFixed(1)} MB`;
  if (n >= 1024) return `${Math.round(n / 1024)} KB`;
  return `${n} B`;
}
