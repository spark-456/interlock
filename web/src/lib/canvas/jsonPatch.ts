// Minimal RFC 6902 JSON Patch (add / replace / remove) for `build.flow_patch` events.

export interface PatchOp {
  op: 'add' | 'replace' | 'remove' | string;
  path: string;
  value?: unknown;
}

type Json = unknown;

function unescape(token: string): string {
  return token.replace(/~1/g, '/').replace(/~0/g, '~');
}

/** Applies ops to a deep copy of `doc` and returns it. Unknown ops are ignored. */
export function applyPatch<T>(doc: T, ops: PatchOp[]): T {
  const root = structuredClone(doc) as Json;
  for (const op of ops) apply(root, op);
  return root as T;
}

function apply(root: Json, op: PatchOp): void {
  const tokens = op.path.split('/').slice(1).map(unescape);
  if (tokens.length === 0) return;
  let parent = root as Record<string, Json> | Json[];
  for (const t of tokens.slice(0, -1)) {
    const next = Array.isArray(parent) ? parent[Number(t)] : parent[t];
    if (next === undefined || next === null || typeof next !== 'object') {
      if (op.op !== 'add' || Array.isArray(parent)) return;
      (parent as Record<string, Json>)[t] = {};
    }
    parent = (Array.isArray(parent) ? parent[Number(t)] : parent[t]) as Record<string, Json> | Json[];
  }
  const last = tokens[tokens.length - 1]!;
  if (Array.isArray(parent)) {
    const i = last === '-' ? parent.length : Number(last);
    if (op.op === 'add') parent.splice(i, 0, structuredClone(op.value));
    else if (op.op === 'replace') parent[i] = structuredClone(op.value);
    else if (op.op === 'remove') parent.splice(i, 1);
  } else {
    if (op.op === 'add' || op.op === 'replace') parent[last] = structuredClone(op.value);
    else if (op.op === 'remove') delete parent[last];
  }
}
