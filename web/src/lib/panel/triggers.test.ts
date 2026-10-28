import { describe, expect, it } from 'vitest';
import { describeCron } from './triggers';

describe('describeCron', () => {
  it('reads the common schedules in plain words', () => {
    expect(describeCron('0 9 * * MON')).toBe('Every Monday at 09:00');
    expect(describeCron('30 16 * * FRI')).toBe('Every Friday at 16:30');
    expect(describeCron('0 8 * * 1-5')).toBe('Every weekday at 08:00');
    expect(describeCron('0 7 * * *')).toBe('Every day at 07:00');
    expect(describeCron('0 * * * *')).toBe('Every hour, on the hour');
    expect(describeCron('*/15 * * * *')).toBe('Every 15 minutes');
    expect(describeCron('0 9 * * 1,3')).toBe('Every Monday and Wednesday at 09:00');
  });

  it('leaves unusual schedules alone', () => {
    expect(describeCron('0 9 1 * *')).toBeNull();
    expect(describeCron('nonsense')).toBeNull();
  });
});
