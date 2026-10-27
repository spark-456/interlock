<script lang="ts">
  // One settings field, chosen by its schema kind and `x-widget` hint.
  import type { Flow, JsonValue } from '$lib/api/types';
  import Button from '$lib/design/Button.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import Input from '$lib/design/Input.svelte';
  import SegmentedControl from '$lib/design/SegmentedControl.svelte';
  import Select from '$lib/design/Select.svelte';
  import Toggle from '$lib/design/Toggle.svelte';
  import ExpressionInput from './ExpressionInput.svelte';
  import SchemaForm from './SchemaForm.svelte';
  import type { Source } from './mapper';
  import { branchOf, emptyOf, type Field, type Schema } from './schema';

  let {
    field,
    value,
    onchange,
    sources,
    flow,
    errors,
    path
  }: {
    field: Field;
    value: JsonValue;
    onchange: (v: JsonValue) => void;
    sources: Source[];
    flow: Flow;
    errors: Record<string, string>;
    path: string;
  } = $props();

  const s = $derived(field.schema);
  const id = $derived(`f-${path.replace(/\W/g, '-')}`);
  const label = $derived(field.required ? `${field.label} *` : field.label);
  const error = $derived(errors[path]);
  const text = $derived(typeof value === 'string' ? value : value == null ? '' : String(value));
  const list = $derived(Array.isArray(value) ? value : []);
  const ROLES = [
    { value: 'architect', label: 'Most capable' },
    { value: 'agent', label: 'Balanced' },
    { value: 'fast', label: 'Fastest' }
  ];
  const nice = (v: JsonValue) => String(v).replace(/([a-z])([A-Z])/g, '$1 $2').replace(/^./, (c) => c.toUpperCase());
  const without = (i: number) => list.filter((_, j) => j !== i);
</script>

{#if field.kind === 'expression' || field.kind === 'longtext' || field.kind === 'file'}
  <ExpressionInput
    {id}
    {label}
    value={text}
    {onchange}
    {sources}
    {flow}
    multiline={field.kind === 'longtext'}
    {error}
    hint={s.description ?? (field.kind === 'file' ? 'A file from an earlier step, e.g. {{ item.files.video }}' : undefined)}
  />
{:else if field.kind === 'text'}
  <Input {id} {label} bind:value={() => text, (v) => onchange(v)} {error} hint={s.description} />
{:else if field.kind === 'number'}
  <Input
    {id}
    {label}
    type="number"
    min={s.minimum}
    max={s.maximum}
    step={s.type === 'integer' ? 1 : 'any'}
    bind:value={() => text, (v) => onchange(v === '' ? null : Number(v))}
    {error}
    hint={s.description}
  />
{:else if field.kind === 'select'}
  <Select
    {id}
    {label}
    options={[...(field.required ? [] : [{ value: '', label: '—' }]), ...(s.enum ?? []).map((v) => ({ value: String(v), label: nice(v) }))]}
    bind:value={() => text, (v) => onchange(v === '' ? null : v)}
    {error}
  />
{:else if field.kind === 'model-role'}
  <SegmentedControl label={field.label} options={ROLES} bind:value={() => text || 'agent', (v) => onchange(v)} />
{:else if field.kind === 'boolean'}
  <Toggle label={field.label} bind:checked={() => value === true, (v) => onchange(v)} />
{:else if field.kind === 'multiselect'}
  <fieldset>
    <legend>{label}</legend>
    <div class="chips">
      {#each s.items?.enum ?? [] as opt (String(opt))}
        {@const on = list.includes(opt)}
        <Chip kind="filter" selected={on} onclick={() => onchange(on ? list.filter((v) => v !== opt) : [...list, opt])}>{nice(opt)}</Chip>
      {/each}
    </div>
    {#if error}<span class="err">{error}</span>{/if}
  </fieldset>
{:else if field.kind === 'list'}
  <fieldset>
    <legend>{label}</legend>
    {#each list as item, i (i)}
      <div class="row">
        <Input aria-label="{field.label} {i + 1}" bind:value={() => String(item ?? ''), (v) => onchange(list.map((x, j) => (j === i ? v : x)))} />
        <IconButton icon="x" label="Remove {field.label} {i + 1}" onclick={() => onchange(without(i))} />
      </div>
    {/each}
    <div><Button size="s" variant="ghost" icon="plus" onclick={() => onchange([...list, ''])}>Add</Button></div>
    {#if error}<span class="err">{error}</span>{/if}
  </fieldset>
{:else if field.kind === 'objects'}
  <fieldset>
    <legend>{label}</legend>
    {#each list as item, i (i)}
      <div class="card">
        <div class="card-head">
          <span>{s.items?.title ?? field.label} {i + 1}</span>
          <IconButton icon="x" label="Remove {s.items?.title ?? field.label} {i + 1}" onclick={() => onchange(without(i))} />
        </div>
        <SchemaForm
          schema={s.items as Schema}
          value={item}
          onchange={(v) => onchange(list.map((x, j) => (j === i ? v : x)))}
          {sources}
          {flow}
          {errors}
          path="{path}.{i}"
        />
      </div>
    {/each}
    <div><Button size="s" variant="ghost" icon="plus" onclick={() => onchange([...list, emptyOf(s.items as Schema)])}>Add {(s.items?.title ?? 'item').toLowerCase()}</Button></div>
    {#if error}<span class="err">{error}</span>{/if}
  </fieldset>
{:else if field.kind === 'object'}
  <fieldset>
    <legend>{label}</legend>
    <SchemaForm schema={s} {value} {onchange} {sources} {flow} {errors} {path} />
  </fieldset>
{:else if field.kind === 'oneOf'}
  {@const branches = s.oneOf ?? []}
  {@const at = branchOf(s, value)}
  <fieldset>
    <legend>{label}</legend>
    <Select
      label="Kind"
      options={branches.map((b, i) => ({ value: String(i), label: b.title ?? `Option ${i + 1}` }))}
      bind:value={() => String(at), (v) => onchange(emptyOf(branches[Number(v)]!))}
    />
    <SchemaForm schema={branches[at]!} {value} {onchange} {sources} {flow} {errors} {path} />
  </fieldset>
{/if}

<style>
  fieldset { border: 0; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; min-width: 0; }
  legend { padding: 0 0 6px; font-size: 13px; font-weight: 600; color: var(--text); }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .row { display: flex; align-items: center; gap: 4px; }
  .row :global(.field) { flex: 1; }
  .card { padding: 12px; border: 1px solid var(--border); border-radius: var(--r-card); background: var(--surface-2); display: flex; flex-direction: column; gap: 10px; }
  .card-head { display: flex; align-items: center; justify-content: space-between; font-size: 12px; font-weight: 600; color: var(--text-3); }
  .err { font-size: 12px; color: var(--error-text); }
</style>
