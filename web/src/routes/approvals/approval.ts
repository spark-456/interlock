// What an approval card lets a person edit: the text that will be sent (an agent's answer, a post,
// a reply) rather than the whole item, which carries the agent's activity and sources.
import type { JsonValue } from '$lib/api/types';

/** Fields that hold the text a step sends, most likely first. */
const TEXT_FIELDS = ['answer', 'body', 'text', 'post', 'reply', 'message', 'draft'];

export type Editable = { field: string | null; text: string };

/** The payload's text to show and edit: a string payload itself, or its first text field; null when it has none. */
export function editableText(payload: JsonValue | null): Editable | null {
  if (typeof payload === 'string') return { field: null, text: payload };
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) return null;
  const field = TEXT_FIELDS.find((f) => typeof payload[f] === 'string' && payload[f] !== '');
  return field ? { field, text: payload[field] as string } : null;
}

/** The payload with the edited text put back. */
export function withText(payload: JsonValue | null, edit: Editable, text: string): JsonValue {
  if (edit.field === null || !payload || typeof payload !== 'object' || Array.isArray(payload)) return text;
  return { ...payload, [edit.field]: text };
}
