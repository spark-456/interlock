// Plain words for a trigger step: how it starts the flow.

const DAYS: Record<string, string> = {
  '0': 'Sunday', '1': 'Monday', '2': 'Tuesday', '3': 'Wednesday', '4': 'Thursday', '5': 'Friday', '6': 'Saturday', '7': 'Sunday',
  SUN: 'Sunday', MON: 'Monday', TUE: 'Tuesday', WED: 'Wednesday', THU: 'Thursday', FRI: 'Friday', SAT: 'Saturday'
};

const time = (h: string, m: string) => `${h.padStart(2, '0')}:${m.padStart(2, '0')}`;

/** "Every Monday at 09:00" for the common cron shapes; `null` when it's something more unusual. */
export function describeCron(cron: string): string | null {
  const parts = cron.trim().split(/\s+/);
  if (parts.length !== 5) return null;
  const [m, h, dom, mon, dow] = parts as [string, string, string, string, string];
  const num = /^\d+$/;
  if (dom !== '*' || mon !== '*') return null;
  const every = m.match(/^\*\/(\d+)$/);
  if (every && h === '*' && dow === '*') return `Every ${every[1]} minutes`;
  if (num.test(m) && h === '*' && dow === '*') return m === '0' ? 'Every hour, on the hour' : `Every hour at ${m} past`;
  if (!num.test(m) || !num.test(h)) return null;
  const at = time(h, m);
  if (dow === '*') return `Every day at ${at}`;
  if (dow === '1-5' || dow.toUpperCase() === 'MON-FRI') return `Every weekday at ${at}`;
  const days = dow.toUpperCase().split(',').map((d) => DAYS[d]);
  if (days.some((d) => !d)) return null;
  return `Every ${days.join(' and ')} at ${at}`;
}
