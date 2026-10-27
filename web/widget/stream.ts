/** A source the answer cites, sent as `sources` on a stream chunk. */
export interface Source {
  title?: string;
  url?: string;
}

/** One chat turn, in the OpenAI message shape. */
export interface Message {
  role: 'user' | 'assistant';
  content: string;
}

/** Answer text and sources carried by one stream chunk. */
export interface Chunk {
  text: string;
  sources: Source[];
}

/** Splits buffered SSE text into complete `data:` payloads and the unfinished tail. */
export function splitEvents(buffer: string): { events: string[]; rest: string } {
  const parts = buffer.split(/\r?\n\r?\n/);
  const rest = parts.pop() ?? '';
  const events = parts
    .map((part) =>
      part
        .split(/\r?\n/)
        .filter((line) => line.startsWith('data:'))
        .map((line) => line.slice(5).trimStart())
        .join('\n')
    )
    .filter(Boolean);
  return { events, rest };
}

/** Reads an OpenAI chat-completion chunk; `null` marks the end of the stream. */
export function parseChunk(data: string): Chunk | null {
  if (data === '[DONE]') return null;
  const json = JSON.parse(data) as { choices?: { delta?: { content?: string } }[]; sources?: Source[] };
  return { text: json.choices?.[0]?.delta?.content ?? '', sources: json.sources ?? [] };
}

/** Streams an answer from the widget endpoint, calling `onChunk` as text and sources arrive. */
export async function chat(endpoint: string, model: string, messages: Message[], onChunk: (chunk: Chunk) => void) {
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model, messages, stream: true })
  });
  if (!response.ok || !response.body) throw new Error(`The assistant is unavailable (${response.status}).`);
  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
  let buffer = '';
  for (;;) {
    const { value, done } = await reader.read();
    if (done) return;
    const { events, rest } = splitEvents(buffer + value);
    buffer = rest;
    for (const event of events) {
      const chunk = parseChunk(event);
      if (!chunk) return;
      onChunk(chunk);
    }
  }
}
