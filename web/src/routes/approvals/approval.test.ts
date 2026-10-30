import { describe, expect, it } from 'vitest';
import { editableText, withText } from './approval';

describe('approval text', () => {
  it('edits the answer of an agent item, keeping the rest', () => {
    const item = { message: 'I want to cancel', answer: 'Sorry to hear that.', activity: [{ tool: 'search' }] };
    const edit = editableText(item)!;
    expect(edit).toEqual({ field: 'answer', text: 'Sorry to hear that.' });
    expect(withText(item, edit, 'We are sorry.')).toEqual({ ...item, answer: 'We are sorry.' });
  });

  it('edits a text payload as it is', () => {
    const edit = editableText('Hello')!;
    expect(withText('Hello', edit, 'Hi')).toBe('Hi');
  });

  it('has nothing to edit as text in other payloads', () => {
    expect(editableText({ count: 3 })).toBeNull();
    expect(editableText(null)).toBeNull();
  });
});
