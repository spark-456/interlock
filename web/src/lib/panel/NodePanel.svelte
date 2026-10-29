<script lang="ts">
  // Node panel (ED-3, design inspector): Settings from the manifest's JSON Schema, Input and Output
  // previews from the last run, and "Run this step" with the inputs that run produced.
  import type { ConnectionSummary, Flow, Item, Node, NodeManifest } from '$lib/api/types';
  import { iconFor, toneFor, toneVars } from '$lib/canvas/categories';
  import type { RunState } from '$lib/canvas/runState';
  import AppTile from '$lib/design/AppTile.svelte';
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import Input from '$lib/design/Input.svelte';
  import Tabs from '$lib/design/Tabs.svelte';
  import Toggle from '$lib/design/Toggle.svelte';
  import SchemaForm from '$lib/forms/SchemaForm.svelte';
  import { sources } from '$lib/forms/mapper';
  import { problems, type Schema } from '$lib/forms/schema';
  import FriendlyError from '$lib/errors/FriendlyError.svelte';
  import CredentialPicker from './CredentialPicker.svelte';
  import Preview from './Preview.svelte';
  import TriggerInfo from './TriggerInfo.svelte';

  let {
    flow,
    node,
    manifest,
    run,
    connections,
    dirty,
    onchange,
    onclose,
    onreplace,
    ondelete,
    onrunstep
  }: {
    flow: Flow;
    node: Node;
    manifest: NodeManifest | undefined;
    run: RunState;
    connections: ConnectionSummary[];
    /** Unsaved edits: a step runs with the saved flow, so it has to be saved first. */
    dirty: boolean;
    onchange: (n: Node) => void;
    onclose: () => void;
    onreplace: () => void;
    ondelete: () => void;
    onrunstep: (inputs: Item[]) => void;
  } = $props();

  let tab = $state('settings');
  const schema = $derived((manifest?.settingsSchema ?? {}) as Schema);
  const errors = $derived(problems(schema, node.settings));
  const issues = $derived(Object.keys(errors).length);
  const mapped = $derived(sources(flow, node.id, run.items));
  const inputs = $derived(
    flow.edges.filter((e) => e.to.node === node.id).flatMap((e) => run.items[`${e.from.node}:${e.from.port}`] ?? [])
  );
  const outputs = $derived((manifest?.outputs ?? []).map((p) => ({ ...p, key: `${node.id}:${p.name}` })));
  const running = $derived(run.nodes[node.id]?.status === 'building');
  const failure = $derived(run.nodes[node.id]?.error);
  const tone = $derived(toneFor(node.type));
  const vars = $derived(toneVars(tone));
</script>

<aside class="panel" aria-label="Step settings" data-testid="node-panel">
  <header>
    {#if tone === 'app' && manifest?.app}
      <AppTile name={manifest.app} size={40} />
    {:else}
      <span class="tile" style:--tile={vars.tile} style:border-color={vars.edge}>
        <Icon name={iconFor(node.type, manifest?.icon)} size={20} color={vars.icon} />
      </span>
    {/if}
    <div class="titles">
      <h2>{node.name}</h2>
      <div class="type">{manifest?.title ?? node.type}</div>
    </div>
    <IconButton icon="swap" label="Replace step" onclick={onreplace} />
    <IconButton icon="trash" label="Delete step" onclick={ondelete} />
    <IconButton icon="x" label="Close panel" onclick={onclose} />
  </header>
  <div class="tabs">
    <Tabs
      label="Step sections"
      tabs={[{ value: 'settings', label: 'Settings' }, { value: 'input', label: 'Input' }, { value: 'output', label: 'Output' }]}
      bind:value={tab}
    />
  </div>
  <div class="body" role="tabpanel">
    {#if failure}<FriendlyError error={failure} />{/if}
    {#if tab === 'settings'}
      <Input label="Name" bind:value={() => node.name, (v) => onchange({ ...node, name: v })} />
      {#if manifest?.trigger}<TriggerInfo {flow} {node} saved={!dirty} />{/if}
      {#if manifest?.credentials.length}
        <CredentialPicker
          specs={manifest.credentials}
          {connections}
          value={node.credentials}
          onchange={(credentials) => onchange({ ...node, credentials })}
        />
      {/if}
      <SchemaForm {schema} value={node.settings} onchange={(settings) => onchange({ ...node, settings })} sources={mapped} {flow} {errors} />
      <Toggle label="Turn this step off" bind:checked={() => node.disabled, (v) => onchange({ ...node, disabled: v })} />
    {:else if tab === 'input'}
      <Preview items={inputs} />
    {:else}
      {#each outputs as p (p.key)}
        {#if outputs.length > 1}<h3>{p.label ?? p.name}</h3>{/if}
        <Preview items={run.items[p.key] ?? []} total={run.counts[p.key]} />
      {/each}
    {/if}
  </div>
  <footer>
    <Button icon="play" loading={running} disabled={manifest?.trigger || dirty} title={dirty ? 'Save first' : undefined} onclick={() => ((tab = 'output'), onrunstep(inputs))}>Run this step</Button>
    {#if issues}<span class="warn">{issues === 1 ? '1 setting needs' : `${issues} settings need`} attention</span>
    {:else if dirty && !manifest?.trigger}<span class="hint">Save to run this step</span>{/if}
  </footer>
</aside>

<style>
  .panel {
    flex: 0 0 400px; width: 400px; height: 100%; display: flex; flex-direction: column; min-height: 0;
    background: var(--surface); border-left: 1px solid var(--border);
  }
  header { display: flex; align-items: center; gap: 8px; padding: 18px 12px 14px 20px; }
  .tile {
    flex: 0 0 40px; width: 40px; height: 40px; border-radius: 11px; border: 1px solid;
    background: linear-gradient(180deg, var(--surface) -40%, var(--tile) 70%); box-shadow: var(--shadow-tile);
    display: flex; align-items: center; justify-content: center;
  }
  .titles { flex: 1 1 auto; min-width: 0; margin-left: 4px; }
  h2 { margin: 0; font-size: 18px; line-height: 24px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .type { font-size: 12px; color: var(--text-3); }
  .tabs { padding: 0 20px; }
  .body { flex: 1 1 auto; min-height: 0; overflow: auto; padding: 16px 20px; display: flex; flex-direction: column; gap: 16px; }
  h3 { margin: 0; font-size: 13px; font-weight: 700; }
  footer { display: flex; align-items: center; gap: 12px; padding: 12px 20px; border-top: 1px solid var(--border); }
  .warn { font-size: 12px; color: var(--warning-text); }
  .hint { font-size: 12px; color: var(--text-3); }
  @media (max-width: 767px) {
    .panel { position: fixed; inset: 0; z-index: 20; width: auto; border-left: 0; }
  }
</style>
