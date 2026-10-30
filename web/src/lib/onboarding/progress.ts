// First-run progress, remembered per user in the browser (Q35). Nothing is stored on the server.

/** What a user has already seen of the onboarding. */
export interface Progress {
  /** The welcome was closed (or the tour started from it). */
  welcomed: boolean;
  /** The getting-started card on Flows was hidden. */
  checklistHidden: boolean;
}

/** A first visit. */
export const FRESH: Progress = { welcomed: false, checklistHidden: false };

/** The storage key for a user, or for the workspace when nobody is signed in. */
export const progressKey = (who: string) => `interlock.onboarding.${who}`;

/** Reads a user's progress; missing, unreadable or blocked storage counts as a first visit. */
export function readProgress(storage: Pick<Storage, 'getItem'> | undefined, who: string): Progress {
  const progress = { ...FRESH };
  try {
    const saved: unknown = JSON.parse(storage?.getItem(progressKey(who)) ?? 'null');
    if (saved && typeof saved === 'object') {
      for (const key of Object.keys(FRESH) as (keyof Progress)[]) {
        const value = (saved as Record<string, unknown>)[key];
        if (typeof value === 'boolean') progress[key] = value;
      }
    }
  } catch {
    // Private windows and blocked site data throw; the onboarding just shows again.
  }
  return progress;
}

/** Saves a user's progress; a failed write only means it shows again next time. */
export function writeProgress(storage: Pick<Storage, 'setItem'> | undefined, who: string, progress: Progress): void {
  try {
    storage?.setItem(progressKey(who), JSON.stringify(progress));
  } catch {
    // See readProgress.
  }
}
