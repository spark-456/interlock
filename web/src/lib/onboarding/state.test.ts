import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { FlowSummary } from '$lib/api';

const flows: FlowSummary[] = [
  { id: 'flw_auto', name: 'Auto', kind: 'automation', version: 1, active: false, updatedAt: '', lastRun: null, apps: [] },
  { id: 'flw_app', name: 'App', kind: 'app', version: 1, active: false, updatedAt: '', lastRun: null, apps: [] }
];
const sample: FlowSummary = { id: 'flw_sample', name: 'Larkspur support agent', kind: 'app', version: 1, active: false, updatedAt: '', lastRun: null, apps: [] };
const listed = vi.fn(async (): Promise<FlowSummary[]> => flows);
const useTemplate = vi.fn(async (id: string) => ({ id: id === 'tpl_support_agent' ? 'flw_sample' : 'flw_other' }));
vi.mock('$lib/api', () => ({ api: { flows: () => listed(), useTemplate: (id: string) => useTemplate(id) } }));

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
    expect(onboarding.tour).toMatchObject({ index: 0, flow: { id: 'flw_app' }, sample: false });
    expect(useTemplate).not.toHaveBeenCalled();
  });

  it('adds the sample app when the workspace has no AI app, so every step has its screen', async () => {
    let added = false;
    listed.mockImplementation(async () => (added ? [flows[0]!, sample] : [flows[0]!]));
    useTemplate.mockImplementationOnce(async (id: string) => ((added = true), { id: id === 'tpl_support_agent' ? 'flw_sample' : 'x' }));
    await startTour();
    expect(useTemplate).toHaveBeenCalledWith('tpl_support_agent');
    expect(onboarding.tour).toMatchObject({ flow: { id: 'flw_sample' }, sample: true });
    listed.mockImplementation(async () => flows);
  });

  it('still starts when the sample cannot be added', async () => {
    listed.mockImplementation(async () => []);
    useTemplate.mockImplementationOnce(async () => {
      throw new Error('offline');
    });
    await startTour();
    expect(onboarding.tour).toMatchObject({ flow: null, sample: false });
    listed.mockImplementation(async () => flows);
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
