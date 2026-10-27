import { describe, expect, it } from 'vitest';
import { isActive, isCanvasRoute } from './nav';

describe('nav', () => {
  it('marks the active section', () => {
    expect(isActive('/flows', '/flows/flw_1/runs/r')).toBe(true);
    expect(isActive('/flows', '/new/bld_1/building')).toBe(true);
    expect(isActive('/new', '/new/bld_1/building')).toBe(false);
    expect(isActive('/settings', '/settingsx')).toBe(false);
  });
  it('collapses the sidebar on canvas screens', () => {
    expect(isCanvasRoute('/new/[id]/building')).toBe(true);
    expect(isCanvasRoute('/flows')).toBe(false);
  });
});
