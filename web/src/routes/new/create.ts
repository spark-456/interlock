// Create screen logic (AR-1): attachments, detected apps and the build request.

import type { Attachment, AttachmentKind, AttachmentRole, BuildRequest, FileRef } from '$lib/api/types';

/** Files above this are refused before upload (design S2 error state). */
export const MAX_BYTES = 50 * 1024 * 1024;

/** One row on the Create screen. `fileId` is set once an upload finishes. */
export interface Draft {
  key: string;
  kind: AttachmentKind;
  name: string;
  role: AttachmentRole;
  detail: string;
  fileId?: string;
  url?: string;
  uploading?: boolean;
  error?: string;
}

export const EXAMPLES = [
  {
    label: 'Weekly product promos',
    icon: 'film',
    goal: 'Every Monday, take our three newest Shopify products, make a short promo video for each with a voiceover, cut it for Reels, TikTok, Shorts and LinkedIn with captions, write the post text, and post once I approve in Slack.'
  },
  {
    label: 'Support inbox replies',
    icon: 'mail',
    goal: 'Reply to emails sent to support@ using our help center and policies. Send simple answers automatically; ask me in Slack before sending anything about refunds or complaints. Log everything in a sheet.'
  },
  {
    label: 'Support chat agent',
    icon: 'chat',
    goal: 'A support chat agent that answers from these documents and can look up invoices through our API.'
  }
];

const size = (bytes: number) =>
  bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`;

const extension = (name: string) => name.split('.').pop()?.toUpperCase() ?? 'File';

let seq = 0;
const nextKey = () => `a${++seq}`;

/** OpenAPI specs are JSON or YAML; knowledge files never are. */
export const isSpec = (f: File) => /\.(json|ya?ml)$/i.test(f.name);

/** A row for a picked file: audio is a voice memo (instructions), other files are knowledge. */
export function fileDraft(f: File): Draft {
  const audio = f.type.startsWith('audio/');
  const draft: Draft = {
    key: nextKey(),
    kind: audio ? 'audio' : 'file',
    name: f.name,
    role: audio ? 'instructions' : 'knowledge',
    detail: `${audio ? 'Voice memo' : extension(f.name)} · ${size(f.size)}`
  };
  if (f.size > MAX_BYTES) draft.error = `${size(f.size)} · too large to read`;
  else draft.uploading = true;
  return draft;
}

/** A row for a pasted link, or an error message when it isn't an http(s) URL. */
export function linkDraft(text: string, kind: 'link' | 'api' = 'link'): Draft | string {
  let url: URL;
  try {
    url = new URL(text.trim());
  } catch {
    return "That doesn't look like a link. Paste a full address starting with https://.";
  }
  if (url.protocol !== 'https:' && url.protocol !== 'http:') return 'Only http and https links work here.';
  return {
    key: nextKey(),
    kind,
    name: url.host + (url.pathname === '/' ? '' : url.pathname),
    role: 'knowledge',
    url: url.href,
    detail: kind === 'api' ? 'OpenAPI spec · endpoints are chosen for your goal' : 'Website · crawled for you'
  };
}

/** Marks a row uploaded. */
export const uploaded = (d: Draft, ref: FileRef): Draft => ({ ...d, uploading: false, fileId: ref.id });

/** Words that also mean an app, besides its own name. */
const ALIASES: Record<string, string[]> = { gmail: ['email', 'inbox'], sheets: ['spreadsheet'], youtube: ['shorts'], instagram: ['reels'] };

/** Apps the goal mentions, from the catalog's app keys (in catalog order, no repeats). */
export function detectApps(goal: string, apps: string[]): string[] {
  const text = goal.toLowerCase();
  return [...new Set(apps)].filter((app) =>
    [app, ...(ALIASES[app] ?? [])].some((w) => new RegExp(`\\b${w.replace(/s$/, '')}`).test(text))
  );
}

/** The request for `POST /api/builds`. Rows still uploading or with errors are left out. */
export function buildRequest(goal: string, drafts: Draft[], hints: string[]): BuildRequest {
  const attachments: Attachment[] = drafts
    .filter((d) => !d.error && !d.uploading)
    .map((d) => ({ kind: d.kind, name: d.name, role: d.role, fileId: d.fileId ?? null, url: d.url ?? null }));
  return { goal: goal.trim(), kind: null, attachments, hints };
}
