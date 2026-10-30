// "@" mentions in Ask Interlock: typing @ and part of a step's name picks that step to change.

export type Step = { id: string; name: string };

/** The text after a trailing "@", or null when the person isn't typing a mention. */
export function mentionQuery(text: string): string | null {
  const m = /(?:^|\s)@([^@\s]*(?: [^@\s]+)*)$/.exec(text);
  return m ? (m[1] ?? '') : null;
}

/** Steps whose name contains `query` (any case), not already picked, best first; at most six. */
export function matchSteps(steps: Step[], query: string, picked: string[]): Step[] {
  const q = query.toLowerCase();
  return steps
    .filter((s) => !picked.includes(s.id) && s.name.toLowerCase().includes(q))
    .sort((a, b) => Number(!a.name.toLowerCase().startsWith(q)) - Number(!b.name.toLowerCase().startsWith(q)))
    .slice(0, 6);
}

/** `text` without its trailing mention. */
export function dropMention(text: string): string {
  return text.replace(/(^|\s)@[^@]*$/, '$1');
}
