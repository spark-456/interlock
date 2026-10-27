// Small display formatters shared by components.

/** `il_live_7c1d…3f9a` → `il_live_••••••3f9a`. */
export function maskSecret(s: string): string {
  const prefix = s.match(/^[a-z]+_[a-z]+_/)?.[0] ?? '';
  return `${prefix}${'•'.repeat(6)}${s.slice(-4)}`;
}

/** Milliseconds → `m:ss` (build timers and log timestamps). */
export function clock(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`;
}

/** USD with three decimals for small amounts (`$0.061`). */
export function usd(v: number): string {
  return `$${v < 1 ? v.toFixed(3) : v.toFixed(2)}`;
}

/** Initials for avatars (`Dana Ruiz` → `DR`). */
export function initials(name: string): string {
  return name.split(/\s+/).filter(Boolean).slice(0, 2).map((w) => w[0]!.toUpperCase()).join('');
}
