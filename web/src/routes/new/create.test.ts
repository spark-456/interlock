import { describe, expect, it } from 'vitest';
import { buildRequest, detectApps, fileDraft, isSpec, linkDraft, MAX_BYTES, type Draft, uploaded } from './create';

const APPS = ['bluesky', 'gmail', 'sheets', 'shopify', 'slack', 'x', 'youtube'];
const file = (name: string, type: string, size: number) => ({ name, type, size }) as File;

describe('create', () => {
  it('detects apps by name, plural and alias', () => {
    expect(detectApps('Reply to emails, ask me in Slack, log it in a sheet', APPS)).toEqual(['gmail', 'sheets', 'slack']);
    expect(detectApps('Post our Shopify products as YouTube Shorts', APPS)).toEqual(['shopify', 'youtube']);
    expect(detectApps('Export next week', APPS)).toEqual([]);
  });

  it('makes rows for files, voice memos and oversized files', () => {
    expect(fileDraft(file('Refunds.pdf', 'application/pdf', 2_400_000))).toMatchObject({
      kind: 'file', role: 'knowledge', detail: 'PDF · 2.3 MB', uploading: true
    });
    expect(fileDraft(file('memo.webm', 'audio/webm', 40_000))).toMatchObject({ kind: 'audio', role: 'instructions', detail: 'Voice memo · 39 KB' });
    const big = fileDraft(file('dump.csv', 'text/csv', MAX_BYTES + 1));
    expect(big.error).toBe('50.0 MB · too large to read');
    expect(big.uploading).toBeUndefined();
  });

  it('accepts only http(s) links', () => {
    expect(linkDraft('https://help.larkspur.example/')).toMatchObject({ kind: 'link', name: 'help.larkspur.example', url: 'https://help.larkspur.example/' });
    expect(linkDraft('https://api.example.com/openapi.json', 'api')).toMatchObject({ kind: 'api', name: 'api.example.com/openapi.json' });
    expect(linkDraft('help center')).toMatch(/doesn't look like a link/);
    expect(linkDraft('ftp://files.example.com')).toMatch(/Only http/);
  });

  it('builds the request from finished rows only', () => {
    const done = uploaded(fileDraft(file('a.pdf', 'application/pdf', 10)), { id: 'file_1', mime: 'application/pdf', bytes: 10, name: 'a.pdf', previewUrl: null });
    const rows: Draft[] = [
      { ...done, role: 'instructions' },
      fileDraft(file('b.pdf', 'application/pdf', 10)),
      fileDraft(file('c.pdf', 'application/pdf', MAX_BYTES + 1)),
      linkDraft('https://help.example.com') as Draft
    ];
    expect(buildRequest('  Reply to email  ', rows, ['gmail'])).toEqual({
      goal: 'Reply to email',
      kind: null,
      hints: ['gmail'],
      attachments: [
        { kind: 'file', name: 'a.pdf', role: 'instructions', fileId: 'file_1', url: null },
        { kind: 'link', name: 'help.example.com', role: 'knowledge', fileId: null, url: 'https://help.example.com/' }
      ]
    });
  });
});

describe('Q24 specs', () => {
  it('tells an OpenAPI spec from a knowledge file', () => {
    const file = (name: string) => new File(['{}'], name);
    expect(['api.json', 'openapi.YAML', 'spec.yml'].map((n) => isSpec(file(n)))).toEqual([true, true, true]);
    expect(['faq.pdf', 'notes.md', 'json.txt'].map((n) => isSpec(file(n)))).toEqual([false, false, false]);
  });
});
