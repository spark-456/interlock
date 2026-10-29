import { describe, expect, it } from 'vitest';
import { friendlyError } from './friendly';

describe('friendlyError', () => {
  it('sends missing and refused connections to Connections', () => {
    expect(friendlyError('missing credential `google`')).toEqual({
      message: 'This step needs a Google connection.',
      action: { label: 'Connect Google', href: '/connections' }
    });
    expect(friendlyError('Notion key is missing; add it on Connections').action?.label).toBe('Connect Notion');
    expect(friendlyError('missing Google access token').message).toBe("Google isn't connected yet.");
    expect(friendlyError('HubSpot rejected the key; check it on Connections').message).toMatch(/^HubSpot refused/);
  });

  it('names the file a step is missing', () => {
    expect(friendlyError('video is required').message).toMatch(/^This step needs a video from an earlier step/);
    expect(friendlyError('missing audio file').message).toMatch(/^This step needs an audio/);
    expect(friendlyError('missing video file id').message).toMatch(/needs a video/);
  });

  it('explains busy services, caps, missing model keys and settings', () => {
    expect(friendlyError('retryable: Slack request failed').message).toMatch(/didn't answer in time/);
    expect(friendlyError('spend cap reached').action?.href).toBe('/settings');
    expect(friendlyError('model config: set NEBIUS_API_KEY').message).toMatch(/AI isn't set up/);
    expect(friendlyError('invalid settings: [ValidationIssue { path: "/text", message: "required" }]').message).toBe(
      'Some settings need fixing: text. Open the step to fill them in.'
    );
    expect(friendlyError('channel is required').message).toBe('The “channel” setting is empty. Fill it in, or map it from an earlier step.');
  });

  it('keeps unknown errors, capitalised', () => {
    expect(friendlyError('shopify: product not found')).toEqual({ message: 'Shopify: product not found' });
  });
});
