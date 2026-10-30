import { describe, expect, it } from 'vitest';
import type { FlowSummary } from '$lib/api';
import { TOUR, place, stepPath } from './tour';

const flow = (id: string, kind: FlowSummary['kind']): FlowSummary => ({
  id, name: id, kind, version: 1, active: false, updatedAt: '2026-10-30T10:00:00Z', lastRun: null, apps: []
});
const step = (page: string) => TOUR.find((s) => s.page === page)!;

describe('tour steps', () => {
  it('covers each part of the app in order and ends where Help is', () => {
    const pages = [...new Set(TOUR.map((s) => s.page))];
    expect(pages).toEqual(['/new', '/flows', '/flows/:flow', '/flows/:flow/chat', '/approvals', '/connections', '/flows/:flow/publish']);
    expect(TOUR.at(-1)?.target).toEqual(['[data-tour="help"]']);
  });

  it('gives every step a title, a body and something to point at', () => {
    for (const s of TOUR) {
      expect(s.title).not.toBe('');
      expect(s.body.length).toBeGreaterThan(40);
      expect(s.target.length).toBeGreaterThan(0);
    }
  });

  it('explains knowledge vs instructions and @ in Refine', () => {
    expect(TOUR.some((s) => /Knowledge.*Instructions/s.test(s.body))).toBe(true);
    expect(TOUR.some((s) => s.title.startsWith('Refine') && s.body.includes('@'))).toBe(true);
  });
});

describe('stepPath', () => {
  it('keeps fixed pages as they are', () => {
    expect(stepPath(step('/new'), null)).toBe('/new');
    expect(stepPath(step('/connections'), flow('f', 'automation'))).toBe('/connections');
  });

  it('fills in the tour flow, or gives null without one', () => {
    expect(stepPath(step('/flows/:flow'), flow('flw_1', 'automation'))).toBe('/flows/flw_1');
    expect(stepPath(step('/flows/:flow/publish'), flow('flw_1', 'app'))).toBe('/flows/flw_1/publish');
    expect(stepPath(step('/flows/:flow'), null)).toBeNull();
  });

  it('shows the playground only for an AI app', () => {
    expect(stepPath(step('/flows/:flow/chat'), flow('flw_1', 'app'))).toBe('/flows/flw_1/chat');
    expect(stepPath(step('/flows/:flow/chat'), flow('flw_1', 'automation'))).toBeNull();
  });
});

describe('place', () => {
  const card = { w: 320, h: 180 };
  const view = { w: 1280, h: 800 };

  it('centres the card without a target', () => {
    expect(place(null, card, view)).toEqual({ x: 480, y: 310 });
  });

  it('puts the card under the target, centred on it', () => {
    expect(place({ x: 500, y: 100, w: 200, h: 40 }, card, view)).toEqual({ x: 440, y: 152 });
  });

  it('puts it above a target near the bottom', () => {
    expect(place({ x: 500, y: 700, w: 200, h: 40 }, card, view)).toEqual({ x: 440, y: 508 });
  });

  it('docks it to the bottom for a target that fills the screen', () => {
    expect(place({ x: 64, y: 64, w: 1216, h: 736 }, card, view)).toEqual({ x: 512, y: 604 });
  });

  it('keeps it on screen next to the edges and on a phone', () => {
    expect(place({ x: 0, y: 10, w: 40, h: 40 }, card, view).x).toBe(16);
    expect(place({ x: 1260, y: 10, w: 20, h: 40 }, card, view).x).toBe(1280 - 320 - 16);
    const phone = place({ x: 0, y: 200, w: 375, h: 100 }, { w: 343, h: 220 }, { w: 375, h: 812 });
    expect(phone).toEqual({ x: 16, y: 312 });
  });
});
