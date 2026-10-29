// Engine and connector errors → a plain message and the next step to take (PRD §8.4 run view).

import { brandFor } from '$lib/design/brands';

export interface Friendly {
  message: string;
  action?: { label: string; href: string };
}

const CONNECTIONS = '/connections';

/** "Google" for `google`, "Notion" for `notion`; unknown keys are capitalised. */
function appName(key: string): string {
  return brandFor(key)?.title ?? key.charAt(0).toUpperCase() + key.slice(1);
}

/** Setting names from a `ValidationIssue` list printed by the engine, e.g. `path: "/text"`. */
function settingNames(raw: string): string[] {
  return [...raw.matchAll(/path: "\/?([^"]*)"/g)].map((m) => m[1] || 'settings');
}

type Rule = [RegExp, (m: RegExpMatchArray, raw: string) => Friendly];

const RULES: Rule[] = [
  [/missing credential `([\w.-]+)`/, (m) => {
    const app = appName(m[1]!);
    return { message: `This step needs a ${app} connection.`, action: { label: `Connect ${app}`, href: CONNECTIONS } };
  }],
  [/(\w+) key is missing; add it on Connections|missing (\w+) access token/i, (m) => {
    const app = (m[1] ?? m[2])!;
    return { message: `${app} isn't connected yet.`, action: { label: `Connect ${app}`, href: CONNECTIONS } };
  }],
  [/(?:NEBIUS|OPENROUTER|GROQ)_API_KEY|model config|no model provider/i, () => ({
    message: "AI isn't set up on this server yet. Add a model key (NEBIUS_API_KEY, or a free OPENROUTER_API_KEY) and restart it."
  })],
  [/(\w+) rejected the (?:API )?key|invalid_grant|token (?:has )?expired|HTTP 401/i, (m) => ({
    message: `${m[1] ?? 'The app'} refused the saved connection. Reconnect it, then run again.`,
    action: { label: 'Open Connections', href: CONNECTIONS }
  })],
  [/spend(?:ing)? cap reached/i, () => ({
    message: 'This run reached its spending cap, so it stopped instead of spending more. Raise the cap or try again tomorrow.',
    action: { label: 'Open Settings', href: '/settings' }
  })],
  [/^retryable:|rate limited|is busy|HTTP 429|provider unavailable|timed out/i, () => ({
    message: "The service didn't answer in time, even after retries. Run it again in a minute."
  })],
  [/\b(video|audio|image|file|attachment|clip)s? (?:file )?is required|missing (?:(\w+) )?file/i, (m) => {
    const kind = (m[1] ?? m[2] ?? 'file').toLowerCase();
    const a = /^[aeiou]/.test(kind) ? 'an' : 'a';
    return { message: `This step needs ${a} ${kind} from an earlier step. Connect a step that makes one, or map it in Settings.` };
  }],
  [/tool \S+ is unavailable/i, () => ({
    message: "The agent asked for a tool this flow doesn't have. Add that tool to the flow or take it off the agent's list."
  })],
  [/ffmpeg|ffprobe|caption font/i, () => ({
    message: "The video tools (ffmpeg) aren't installed on this server, so media steps can't run here."
  })],
  [/is not implemented yet/, () => ({ message: "This kind of step isn't available yet. Replace it with another step." })],
  [/^invalid settings/, (_m, raw) => {
    const names = settingNames(raw);
    return { message: `Some settings need fixing${names.length ? `: ${names.join(', ')}` : ''}. Open the step to fill them in.` };
  }],
  [/^(?:missing (\w+)|(\w+) is required|(\w+) must not be empty)$/, (m) => ({
    message: `The “${m[1] ?? m[2] ?? m[3]}” setting is empty. Fill it in, or map it from an earlier step.`
  })]
];

/** A plain-language version of `raw`; unknown errors keep their text, starting with a capital. */
export function friendlyError(raw: string): Friendly {
  const text = raw.trim();
  for (const [pattern, make] of RULES) {
    const m = text.match(pattern);
    if (m) return make(m, text);
  }
  return { message: text.charAt(0).toUpperCase() + text.slice(1) };
}
