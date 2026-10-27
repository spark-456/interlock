<script lang="ts">
  // Settings form rendered from a node's JSON Schema (ED-3): no per-node UI code. Recursive for
  // objects, lists of objects and oneOf branches.
  import type { Flow, JsonValue } from '$lib/api/types';
  import Field from './Field.svelte';
  import type { Source } from './mapper';
  import { fieldsOf, type Schema } from './schema';

  let {
    schema,
    value,
    onchange,
    sources,
    flow,
    errors = {},
    path = ''
  }: {
    schema: Schema;
    value: JsonValue;
    onchange: (v: Record<string, JsonValue>) => void;
    sources: Source[];
    flow: Flow;
    errors?: Record<string, string>;
    path?: string;
  } = $props();

  const obj = $derived((value && typeof value === 'object' && !Array.isArray(value) ? value : {}) as Record<string, JsonValue>);
</script>

<div class="form">
  {#each fieldsOf(schema) as f (f.key)}
    <Field
      field={f}
      value={obj[f.key] ?? null}
      onchange={(v) => onchange({ ...obj, [f.key]: v })}
      {sources}
      {flow}
      {errors}
      path={path ? `${path}.${f.key}` : f.key}
    />
  {/each}
</div>

<style>
  .form { display: flex; flex-direction: column; gap: 16px; }
</style>
