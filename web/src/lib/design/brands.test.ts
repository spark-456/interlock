import { describe, expect, it } from 'vitest';
import { brandFor } from './brands';

describe('brandFor', () => {
  it('finds marks by app key, display name and partner label', () => {
    expect(brandFor('gmail')?.title).toBe('Gmail');
    expect(brandFor('Google Sheets')?.title).toBe('Google Sheets');
    expect(brandFor('sheets')?.title).toBe('Google Sheets');
    expect(brandFor('Claude Code')?.title).toBe('Claude');
    expect(brandFor('OpenAI SDK')?.hex).toMatch(/^[0-9A-F]{6}$/);
    for (const b of ['slack', 'discord', 'telegram', 'bluesky', 'x', 'instagram', 'tiktok', 'shopify', 'youtube', 'nvidia', 'notion', 'airtable', 'hubspot']) {
      expect(brandFor(b)?.path.length, b).toBeGreaterThan(20);
    }
  });

  it('has no mark for brands Simple Icons no longer ships', () => {
    expect(brandFor('LinkedIn')).toBeUndefined();
    expect(brandFor('VS Code')).toBeUndefined();
  });
});
