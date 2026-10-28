// JSON Schema → form fields (ED-3). Covers what node manifests use: string, number, integer,
// boolean, enum, array, object and oneOf, with `x-widget` hints for how a string is edited.

import type { JsonValue } from '$lib/api/types';

export interface Schema {
  type?: string;
  title?: string;
  description?: string;
  default?: JsonValue;
  enum?: JsonValue[];
  minimum?: number;
  maximum?: number;
  properties?: Record<string, Schema>;
  required?: string[];
  items?: Schema;
  oneOf?: Schema[];
  'x-widget'?: string;
}

export type Kind =
  | 'text' | 'longtext' | 'expression' | 'file' | 'select' | 'model-role' | 'number' | 'boolean'
  | 'multiselect' | 'list' | 'objects' | 'object' | 'oneOf' | 'json';

/** How a schema is edited. */
export function kindOf(s: Schema): Kind {
  if (s.oneOf?.length) return 'oneOf';
  const w = s['x-widget'];
  if (w === 'model-role') return 'model-role';
  if (w === 'json') return 'json';
  if (s.enum || w === 'select') return 'select';
  if (!s.type && s.properties) return 'object';
  switch (s.type) {
    case 'number':
    case 'integer': return 'number';
    case 'boolean': return 'boolean';
    case 'object': return 'object';
    case 'array': return s.items?.enum ? 'multiselect' : s.items?.type === 'object' ? 'objects' : 'list';
  }
  return w === 'longtext' || w === 'expression' || w === 'file' ? w : 'text';
}

export interface Field {
  key: string;
  label: string;
  schema: Schema;
  kind: Kind;
  required: boolean;
}

/** One field per property, in schema order. */
export function fieldsOf(s: Schema): Field[] {
  const required = new Set(s.required ?? []);
  return Object.entries(s.properties ?? {}).map(([key, schema]) => ({
    key, schema, kind: kindOf(schema), required: required.has(key), label: schema.title ?? key
  }));
}

/** A starting value for a schema: its default, else an empty value of its type. */
export function emptyOf(s: Schema): JsonValue {
  if (s.default !== undefined) return structuredClone(s.default);
  switch (kindOf(s)) {
    case 'object': return Object.fromEntries(fieldsOf(s).map((f) => [f.key, emptyOf(f.schema)]));
    case 'multiselect':
    case 'list':
    case 'objects': return [];
    case 'boolean': return false;
    case 'number': return null;
    default: return '';
  }
}

/** The oneOf branch a value matches best: the one with the most required keys, all present. */
export function branchOf(s: Schema, value: JsonValue): number {
  const obj = (value ?? {}) as Record<string, JsonValue>;
  const score = (b: Schema) => {
    const req = b.required ?? [];
    return req.every((k) => obj[k] !== undefined && obj[k] !== '') ? req.length : -1;
  };
  return (s.oneOf ?? []).reduce((best, b, i, all) => (score(b) > score(all[best]!) ? i : best), 0);
}

/** Missing required values and out-of-range numbers, keyed by dotted path. Expressions are not checked. */
export function problems(s: Schema, value: JsonValue, path = ''): Record<string, string> {
  const out: Record<string, string> = {};
  const obj = (value ?? {}) as Record<string, JsonValue>;
  if (kindOf(s) === 'oneOf') return problems(s.oneOf![branchOf(s, value)]!, value, path);
  for (const f of fieldsOf(s)) {
    const v = obj[f.key];
    const at = path ? `${path}.${f.key}` : f.key;
    if (f.required && (v === undefined || v === null || v === '' || (Array.isArray(v) && v.length === 0))) {
      out[at] = `${f.label} is required`;
    } else if (typeof v === 'number') {
      if (f.schema.minimum !== undefined && v < f.schema.minimum) out[at] = `At least ${f.schema.minimum}`;
      if (f.schema.maximum !== undefined && v > f.schema.maximum) out[at] = `At most ${f.schema.maximum}`;
      if (f.schema.type === 'integer' && !Number.isInteger(v)) out[at] = 'A whole number';
    } else if (f.kind === 'object' && v && typeof v === 'object') {
      Object.assign(out, problems(f.schema, v, at));
    }
  }
  return out;
}

/** Parses typed JSON for a JSON field; an error says what is wrong in plain words. */
export function readJson(text: string): { value: JsonValue } | { error: string } {
  if (!text.trim()) return { value: [] };
  try {
    return { value: JSON.parse(text) as JsonValue };
  } catch {
    return { error: 'Not valid JSON yet: check quotes, commas and brackets.' };
  }
}
