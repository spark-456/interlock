import { afterEach, describe, expect, it, vi } from 'vitest';
import { chat, parseChunk, splitEvents, type Chunk } from './stream';

const chunk = (content: string, extra: object = {}) =>
  `data: ${JSON.stringify({ object: 'chat.completion.chunk', choices: [{ index: 0, delta: { content } }], ...extra })}\n\n`;

function respond(parts: string[], status = 200) {
  const body = new ReadableStream<Uint8Array>({
    start(controller) {
      for (const part of parts) controller.enqueue(new TextEncoder().encode(part));
      controller.close();
    }
  });
  const fetch = vi.fn(async (_url: string, _init?: RequestInit) => new Response(body, { status }));
  vi.stubGlobal('fetch', fetch);
  return fetch;
}

afterEach(() => vi.unstubAllGlobals());

describe('widget stream', () => {
  it('splits SSE text and keeps the unfinished tail', () => {
    expect(splitEvents('data: a\n\ndata: b\r\n\r\ndata: c')).toEqual({ events: ['a', 'b'], rest: 'data: c' });
    expect(parseChunk('[DONE]')).toBeNull();
    expect(parseChunk('{"choices":[{"delta":{}}],"sources":[{"title":"FAQ"}]}')).toEqual({ text: '', sources: [{ title: 'FAQ' }] });
  });

  it('streams text and sources across split network reads, stopping at [DONE]', async () => {
    const whole = chunk('Hel') + chunk('lo', { sources: [{ title: 'Guide', url: 'https://x.test/g' }] }) + 'data: [DONE]\n\n' + chunk('ignored');
    const fetch = respond([whole.slice(0, 17), whole.slice(17, 90), whole.slice(90)]);
    const seen: Chunk[] = [];
    await chat('https://x.test/w/app/chat', 'app', [{ role: 'user', content: 'Hi' }], (c) => seen.push(c));
    expect(seen.map((c) => c.text).join('')).toBe('Hello');
    expect(seen.at(-1)?.sources).toEqual([{ title: 'Guide', url: 'https://x.test/g' }]);
    expect(JSON.parse(String(fetch.mock.calls[0]?.[1]?.body))).toEqual({ model: 'app', messages: [{ role: 'user', content: 'Hi' }], stream: true });
  });

  it('reports an unavailable endpoint', async () => {
    respond([], 404);
    await expect(chat('https://x.test/w/app/chat', 'app', [], () => {})).rejects.toThrow('unavailable (404)');
  });
});
