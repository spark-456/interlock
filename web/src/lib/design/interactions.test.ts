import { describe, expect, it } from 'vitest';
import { bytes, menuMove, pickFiles } from './interactions';

const file = (name: string, type: string, size: number) => new File([new Uint8Array(size)], name, { type });

describe('design interactions', () => {
  it('moves through a menu with the arrow keys, wrapping, and jumps with Home and End', () => {
    expect(menuMove(0, 'ArrowDown', 3)).toBe(1);
    expect(menuMove(2, 'ArrowDown', 3)).toBe(0);
    expect(menuMove(0, 'ArrowUp', 3)).toBe(2);
    expect(menuMove(1, 'Home', 3)).toBe(0);
    expect(menuMove(1, 'End', 3)).toBe(2);
    expect(menuMove(1, 'a', 3)).toBeNull();
    expect(menuMove(0, 'ArrowDown', 0)).toBeNull();
  });

  it('accepts files by extension, MIME wildcard and size', () => {
    const files = [file('policy.PDF', 'application/pdf', 10), file('memo.m4a', 'audio/mp4', 10), file('big.pdf', 'application/pdf', 3000), file('x.exe', 'application/x-msdownload', 1)];
    const { ok, rejected } = pickFiles(files, { accept: ['.pdf', 'audio/*'], maxBytes: 2048 });
    expect(ok.map((f) => f.name)).toEqual(['policy.PDF', 'memo.m4a']);
    expect(rejected.map((r) => [r.file.name, r.reason])).toEqual([
      ['big.pdf', 'Files must be 2 KB or smaller.'],
      ['x.exe', 'This file type isn’t supported.']
    ]);
    expect(pickFiles(files, {}).ok).toHaveLength(4);
  });

  it('formats sizes', () => {
    expect([bytes(12), bytes(820 * 1024), bytes(2.4 * 1024 * 1024)]).toEqual(['12 B', '820 KB', '2.4 MB']);
  });
});
