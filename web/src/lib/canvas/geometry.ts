// NodeCanvas geometry (design/interlock/DESIGN.md, NodeCanvas).

export const NODE_WIDTH = 264;
const LABEL_OFFSET = 44;
export const MIN_ZOOM = 0.35;
export const MAX_ZOOM = 1.5;

/** Card height: 116 + 22 per stat row (1 → 138, 2 → 160, 3 → 182). */
export function nodeHeight(statRows: number): number {
  return 116 + 22 * Math.max(0, statRows);
}

/** Horizontal bezier from a source's right-middle to a target's left-middle. */
export function bezier(sx: number, sy: number, tx: number, ty: number): string {
  const d = Math.max(48, Math.abs(tx - sx) / 2);
  return `M${sx} ${sy} C${sx + d} ${sy} ${tx - d} ${ty} ${tx} ${ty}`;
}

/** Vertical bezier (bottom-middle → top-middle), used for the test suite hanging under an agent. */
export function verticalBezier(sx: number, sy: number, tx: number, ty: number): string {
  return `M${sx} ${sy} C${sx} ${sy + 24} ${tx} ${ty - 24} ${tx} ${ty}`;
}

export interface ColumnLabel {
  x: number;
  y: number;
  text: string;
  tone: string;
}

/** One label per group, at the group's leftmost x and 44 px above the topmost node. */
export function columnLabels(
  nodes: { group?: string | null; position: { x: number; y: number }; tone: string }[],
  groups: { id: string; label: string }[]
): ColumnLabel[] {
  if (nodes.length === 0) return [];
  const top = Math.min(...nodes.map((n) => n.position.y)) - LABEL_OFFSET;
  const out: ColumnLabel[] = [];
  for (const g of groups) {
    const members = nodes.filter((n) => n.group === g.id);
    if (members.length === 0) continue;
    const first = members.reduce((a, b) => (b.position.y < a.position.y ? b : a));
    out.push({ x: Math.min(...members.map((n) => n.position.x)), y: top, text: g.label, tone: first.tone });
  }
  return out;
}
