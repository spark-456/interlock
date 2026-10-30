import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { FlowSummary } from '$lib/api';

const flows: FlowSummary[] = [
  { id: 'flw_auto', name: 'Auto', kind: 'automation', version: 1, active: false, updatedAt: '', lastRun: null, apps: [] },
  { id: 'flw_app', name: 'App', kind: 'app', version: 1, active: false, updatedAt: '', lastRun: null, apps: [] }
];
vi.mock('$lib/api', () => ({ api: { flows: vi.fn(async () => flows) } }));

const { endTour, moveTour, onboarding, remember, signIn, startTour } = await import('./state.svelte');
const { TOUR } = await import('./tour');

const saved = new Map<string, string>();
vi.stubGlobal('localStorage', { getItem: (k: string) => saved.get(k) ?? null, setItem: (k: string, v: string) => void saved.set(k, v) });

beforeEach(() => {
  saved.clear();
  Object.assign(onboarding, { who: null, welcomeOpen: false, helpOpen: false, tour: null });
});

describe('onboarding state', () => {
  it('greets a person once, and remembers it per user', () => {
    signIn('usr_1');
    expect(onboarding.welcomeOpen).toBe(true);
    onboarding.welcomeOpen = false;
    remember({ welcomed: true });

    signIn('usr_2');
    expect(onboarding.welcomeOpen).toBe(true);

    onboarding.welcomeOpen = false;
    signIn('usr_1');
    expect(onboarding.welcomeOpen).toBe(false);
    expect(onboarding.progress.welcomed).toBe(true);
  });

  it('does not reopen the welcome when the same person is loaded again', () => {
    signIn('usr_1');
    onboarding.welcomeOpen = false;
    signIn('usr_1');
    expect(onboarding.welcomeOpen).toBe(false);
  });

  it('starts the tour on the first AI app and closes the welcome and help', async () => {
    signIn('usr_1');
    onboarding.helpOpen = true;
    await startTour();
    expect(onboarding.welcomeOpen).toBe(false);
    expect(onboarding.helpOpen).toBe(false);
    expect(onboarding.progress.welcomed).toBe(true);
    expect(onboarding.tour).toMatchObject({ index: 0, flow: { id: 'flw_app' } });
  });

  it('moves within the steps and ends after the last one', async () => {
    await startTour();
    moveTour(-1);
    expect(onboarding.tour?.index).toBe(0);
    for (let i = 1; i < TOUR.length; i++) moveTour(1);
    expect(onboarding.tour?.index).toBe(TOUR.length - 1);
    moveTour(1);
    expect(onboarding.tour).toBeNull();
    moveTour(1);
    expect(onboarding.tour).toBeNull();
  });

  it('ends on request', async () => {
    await startTour();
    endTour();
    expect(onboarding.tour).toBeNull();
  });
});
