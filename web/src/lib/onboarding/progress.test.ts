import { describe, expect, it } from 'vitest';
import { FRESH, progressKey, readProgress, writeProgress } from './progress';

function memory(): Storage {
  const data = new Map<string, string>();
  return {
    get length() { return data.size; },
    clear: () => data.clear(),
    key: (i) => [...data.keys()][i] ?? null,
    getItem: (k) => data.get(k) ?? null,
    setItem: (k, v) => void data.set(k, v),
    removeItem: (k) => void data.delete(k)
  };
}

describe('progress', () => {
  it('is a first visit when nothing is saved', () => {
    expect(readProgress(memory(), 'usr_1')).toEqual(FRESH);
    expect(readProgress(undefined, 'usr_1')).toEqual(FRESH);
  });

  it('is remembered per user', () => {
    const s = memory();
    writeProgress(s, 'usr_1', { welcomed: true, checklistHidden: true });
    expect(readProgress(s, 'usr_1')).toEqual({ welcomed: true, checklistHidden: true });
    expect(readProgress(s, 'usr_2')).toEqual(FRESH);
    expect(s.getItem(progressKey('usr_1'))).toBe('{"welcomed":true,"checklistHidden":true}');
  });

  it('ignores broken or foreign data', () => {
    const s = memory();
    s.setItem(progressKey('a'), '{not json');
    s.setItem(progressKey('b'), '{"welcomed":"yes","checklistHidden":true,"extra":1}');
    expect(readProgress(s, 'a')).toEqual(FRESH);
    expect(readProgress(s, 'b')).toEqual({ welcomed: false, checklistHidden: true });
  });

  it('survives storage that throws', () => {
    const blocked = { getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } };
    expect(readProgress(blocked, 'usr_1')).toEqual(FRESH);
    expect(() => writeProgress(blocked, 'usr_1', FRESH)).not.toThrow();
  });
});
