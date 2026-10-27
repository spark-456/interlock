import { describe, expect, it } from 'vitest';
import { clock, initials, maskSecret, usd } from './format';

describe('format', () => {
  it('masks secrets keeping prefix and tail', () => {
    expect(maskSecret('il_live_7c1d0b52e9a43f9a')).toBe('il_live_••••••3f9a');
  });
  it('formats clocks, money and initials', () => {
    expect(clock(88_000)).toBe('1:28');
    expect(clock(112_400)).toBe('1:52');
    expect(usd(0.0612)).toBe('$0.061');
    expect(usd(12.5)).toBe('$12.50');
    expect(initials('Dana Ruiz')).toBe('DR');
  });
});
