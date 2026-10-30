import { describe, expect, it } from 'vitest';
import { dropMention, matchSteps, mentionQuery } from './mentions';

const steps = [
  { id: 'n_agent', name: 'Support agent' },
  { id: 'n_guard', name: 'Safety rules' },
  { id: 'n_slack', name: 'Alert support team' }
];

describe('Ask mentions', () => {
  it('finds the mention being typed', () => {
    expect(mentionQuery('make @saf')).toBe('saf');
    expect(mentionQuery('@')).toBe('');
    expect(mentionQuery('make @safety rul')).toBe('safety rul');
    expect(mentionQuery('email me@work')).toBeNull();
    expect(mentionQuery('no mention')).toBeNull();
  });

  it('matches step names, name starts first, skipping picked ones', () => {
    expect(matchSteps(steps, 'sup', []).map((s) => s.id)).toEqual(['n_agent', 'n_slack']);
    expect(matchSteps(steps, 'sup', ['n_agent']).map((s) => s.id)).toEqual(['n_slack']);
    expect(matchSteps(steps, '', []).length).toBe(3);
  });

  it('drops the typed mention once a step is picked', () => {
    expect(dropMention('be stricter in @saf')).toBe('be stricter in ');
    expect(dropMention('@saf')).toBe('');
  });
});
