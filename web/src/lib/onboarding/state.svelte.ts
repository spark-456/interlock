// Onboarding state shared by the welcome, the tour, the help dialog and the checklist card (Q35).

import { api, type FlowSummary } from '$lib/api';
import { focusFlow } from './checklist';
import { FRESH, readProgress, writeProgress, type Progress } from './progress';
import { TOUR } from './tour';

export const onboarding = $state<{
  /** Whose progress is loaded: the user id, or the workspace id when nobody is signed in. */
  who: string | null;
  progress: Progress;
  welcomeOpen: boolean;
  helpOpen: boolean;
  /** The running tour: its step and the flow it shows. */
  tour: { index: number; flow: FlowSummary | null } | null;
}>({ who: null, progress: { ...FRESH }, welcomeOpen: false, helpOpen: false, tour: null });

const storage = () => (typeof localStorage === 'undefined' ? undefined : localStorage);

/** Loads someone's progress and greets them on their first visit. */
export function signIn(who: string): void {
  if (onboarding.who === who) return;
  onboarding.who = who;
  onboarding.progress = readProgress(storage(), who);
  onboarding.welcomeOpen = !onboarding.progress.welcomed;
}

/** Records progress for the current user. */
export function remember(patch: Partial<Progress>): void {
  onboarding.progress = { ...onboarding.progress, ...patch };
  if (onboarding.who) writeProgress(storage(), onboarding.who, onboarding.progress);
}

/** Starts the tour from its first step, on the workspace's first AI app (else its first flow). */
export async function startTour(): Promise<void> {
  onboarding.welcomeOpen = onboarding.helpOpen = false;
  remember({ welcomed: true });
  const flows = await api.flows().catch(() => []);
  onboarding.tour = { index: 0, flow: focusFlow(flows) };
}

/** Moves the tour one step; past the last step it ends. */
export function moveTour(by: 1 | -1): void {
  const tour = onboarding.tour;
  if (!tour) return;
  const index = tour.index + by;
  onboarding.tour = index >= TOUR.length ? null : { ...tour, index: Math.max(0, index) };
}

/** Ends the tour. */
export function endTour(): void {
  onboarding.tour = null;
}
