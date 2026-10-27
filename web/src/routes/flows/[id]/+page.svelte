<script lang="ts">
  // S6 editor (ED-1, ED-2, ED-6, ED-9): the flow on the canvas, a palette to add or replace steps,
  // undo/redo, tidy, save, a test run with live state from the run stream, and Ask Interlock diffs.
  // The node panel (Settings / Input / Output) is W03's and mounts beside the canvas.
  import { onDestroy } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';
  import { page } from '$app/state';
  import { api, runEvents, type Flow, type NodeManifest } from '$lib/api';
  import AskBar from '$lib/canvas/AskBar.svelte';
  import NodeCanvas from '$lib/canvas/NodeCanvas.svelte';
  import NodeList from '$lib/canvas/NodeList.svelte';
  import Palette from '$lib/canvas/Palette.svelte';
  import {
    COLUMN, addNode, connect, diffFlows, edgeNear, insertOnEdge, moveNodes, record, redo, removeNodes, replaceNode, tidy, undo,
    type History
  } from '$lib/canvas/edit';
  import { NODE_WIDTH, columnLabels, nodeHeight } from '$lib/canvas/geometry';
  import { applyPatch, type PatchOp } from '$lib/canvas/jsonPatch';
  import { idleRun, reduceRun, type RunState } from '$lib/canvas/runState';
  import { flowCards, flowEdges, groupViews } from '$lib/canvas/views';
  import Button from '$lib/design/Button.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import Spinner from '$lib/design/Spinner.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import TopBar from '$lib/design/TopBar.svelte';
  import { toast } from '$lib/design/toasts.svelte';

  const flowId = page.params.id ?? '';
  const speed = Number(page.url.searchParams.get('speed') ?? import.meta.env.VITE_FIXTURE_SPEED ?? 1) || 1;

  let flow = $state.raw<Flow | null>(null);
  let saved = $state.raw<Flow | null>(null);
  let history = $state.raw<History>({ past: [], future: [] });
  let manifests = $state.raw<NodeManifest[]>([]);
  let loadError = $state<string | null>(null);
  let selected = $state<string | null>(null);
  let palette = $state<'add' | 'replace' | null>(null);
  let asking = $state(false);
  let askPending = $state(false);
  let proposal = $state.raw<{ flow: Flow; summary: string } | null>(null);
  let run = $state.raw<RunState>(idleRun());
  let saving = $state(false);
  let listView = $state(false);
  const mobile = new MediaQuery('max-width: 767px');
  let sub: { close(): void } | null = null;

  Promise.all([api.flow(flowId), api.nodes()])
    .then(([f, c]) => {
      flow = saved = f;
      manifests = c.nodes;
      const runId = page.url.searchParams.get('run');
      if (runId) watch(runId);
    })
    .catch((e: Error) => (loadError = e.message));
  onDestroy(() => sub?.close());

  const catalog = $derived(Object.fromEntries(manifests.map((m) => [m.typeId, m])));
  const shown = $derived(proposal?.flow ?? flow);
  const diff = $derived(proposal && flow ? diffFlows(flow, proposal.flow) : undefined);
  const cards = $derived(shown ? flowCards(shown, catalog, run, diff) : []);
  const edges = $derived(shown ? flowEdges(shown, run, diff) : []);
  const labels = $derived(shown ? columnLabels(cards, shown.groups) : []);
  const groups = $derived(shown ? groupViews(shown.groups, cards) : []);
  const dirty = $derived(flow !== saved);
  const touched = $derived(diff ? [...diff.added, ...diff.changed, ...diff.removed.map((n) => n.id)] : []);
  const selectedNode = $derived(flow?.nodes.find((n) => n.id === selected) ?? null);
  const height = (id: string) => nodeHeight(cards.find((c) => c.id === id)?.stats.length ?? 0);
  const port = (nodeId: string, side: 'inputs' | 'outputs') =>
    catalog[flow?.nodes.find((n) => n.id === nodeId)?.type ?? '']?.[side][0]?.name ?? (side === 'inputs' ? 'in' : 'out');

  /** Applies an edit, recording the previous flow for undo. Edits wait while a proposal is shown. */
  function change(next: Flow) {
    if (!flow || proposal || next === flow) return;
    history = record(history, flow);
    flow = next;
  }

  function step(dir: typeof undo) {
    if (!flow || proposal) return;
    const r = dir(history, flow);
    if (r) ({ flow, history } = r);
  }

  /** Where a new step goes: right of the selected step, else after the rightmost one. */
  function slot(): { x: number; y: number } {
    const nodes = flow?.nodes ?? [];
    const rightmost = nodes.reduce<(typeof nodes)[number] | null>((a, n) => (!a || n.position.x > a.position.x ? n : a), null);
    const anchor = nodes.find((n) => n.id === selected) ?? rightmost;
    return anchor ? { x: anchor.position.x + COLUMN, y: anchor.position.y } : { x: 0, y: 0 };
  }

  /** Adds a step; dropped onto an edge, it goes in between the two steps. */
  function add(m: NodeManifest, at = slot()) {
    if (!flow) return;
    const onEdge = edgeNear(flow, { x: at.x + NODE_WIDTH / 2, y: at.y + nodeHeight(0) / 2 }, height);
    const r = (onEdge && insertOnEdge(flow, onEdge, m, at)) || addNode(flow, m, at);
    change(r.flow);
    selected = r.id;
  }

  function pick(m: NodeManifest) {
    if (!flow) return;
    if (palette === 'replace' && selectedNode) {
      change(replaceNode(flow, selectedNode.id, m));
      toast(`Replaced with ${m.title}`, 'success');
      palette = null;
    } else add(m);
  }

  function remove(nodeIds: string[], edgeIds: string[] = []) {
    if (!flow) return;
    const cut = new Set(edgeIds);
    const next = removeNodes(flow, nodeIds);
    change({ ...next, edges: next.edges.filter((e) => !cut.has(e.id)) });
    if (selected && nodeIds.includes(selected)) selected = null;
  }

  async function save() {
    if (!flow) return;
    saving = true;
    try {
      flow = saved = await api.saveFlow(flow);
      toast('Saved', 'success');
    } catch (e) {
      toast((e as Error).message, 'error');
    } finally {
      saving = false;
    }
  }

  async function ask(message: string) {
    if (!flow) return;
    askPending = true;
    try {
      const d = await api.ask(flow.id, message);
      proposal = { flow: applyPatch(flow, d.jsonPatch as unknown as PatchOp[]), summary: d.summary };
    } catch (e) {
      toast((e as Error).message, 'error');
    } finally {
      askPending = false;
    }
  }

  function accept() {
    const next = proposal?.flow;
    proposal = null;
    asking = false;
    if (next) change(next);
  }

  function watch(runId: string) {
    sub?.close();
    run = idleRun();
    sub = runEvents(runId, (e) => (run = reduceRun(run, e)), { speed });
  }

  async function testRun() {
    try {
      watch((await api.testRun(flowId)).runId);
    } catch (e) {
      toast((e as Error).message, 'error');
    }
  }

  function keys(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey) || (e.target as HTMLElement).closest('input, textarea, [contenteditable]')) return;
    const k = e.key.toLowerCase();
    if (k === 'z' || k === 'y') {
      e.preventDefault();
      step(k === 'y' || e.shiftKey ? redo : undo);
    } else if (k === 's') {
      e.preventDefault();
      if (dirty && !proposal) void save();
    }
  }
</script>

<svelte:head><title>{flow?.name ?? 'Editor'} · Interlock</title></svelte:head>
<svelte:window onkeydown={keys} />

<div class="editor">
  <TopBar title={flow?.name ?? 'Loading…'} crumb={{ href: '/flows', label: 'Flows' }} back="/flows" onmenu={() => (listView = !listView)}>
    {#snippet status()}
      {#if run.status === 'running'}<StatusPill tone="building" label="Test run" size={26} />
      {:else if run.status === 'succeeded'}<StatusPill tone="ready" label="Test run passed" size={26} />
      {:else if run.status === 'failed'}<StatusPill tone="failed" label="Test run failed" size={26} />
      {:else if dirty}<StatusPill tone="waiting" label="Unsaved changes" size={26} />{/if}
    {/snippet}
    {#snippet actions()}
      <Button icon="plus" size="s" aria-pressed={palette === 'add'} onclick={() => (palette = palette === 'add' ? null : 'add')}>Add step</Button>
      <Button icon="sparkle" size="s" aria-pressed={asking} onclick={() => (asking = !asking)}>Refine</Button>
      <Button icon="play" size="s" loading={run.status === 'running'} disabled={!flow || dirty} title={dirty ? 'Save first' : undefined} onclick={testRun}>
        Test run
      </Button>
      <Button variant="primary" size="s" loading={saving} disabled={!dirty || !!proposal} onclick={save}>Save</Button>
    {/snippet}
  </TopBar>
  <div class="body">
    {#if palette && manifests.length}
      <Palette
        nodes={manifests}
        replacing={palette === 'replace' ? (selectedNode?.name ?? null) : null}
        onpick={pick}
        onclose={() => (palette = null)}
      />
    {/if}
    <div class="stage" class:list={listView || mobile.current}>
      {#if loadError}
        <p class="state">Couldn't open this flow: {loadError}</p>
      {:else if !flow}
        <p class="state"><Spinner size={16} /> Loading the flow…</p>
      {:else if listView || mobile.current}
        <div class="list-wrap"><NodeList {groups} onselect={(id) => (selected = id)} /></div>
      {:else}
        <NodeCanvas
          {cards}
          {edges}
          {labels}
          bind:selected
          focus={touched}
          editable={!proposal}
          onlist={() => (listView = true)}
          edit={{
            onundo: () => step(undo),
            onredo: () => step(redo),
            ontidy: () => flow && change(tidy(flow, height)),
            canUndo: history.past.length > 0 && !proposal,
            canRedo: history.future.length > 0 && !proposal
          }}
          onmove={(moves) => flow && change(moveNodes(flow, moves))}
          onconnect={(c) => flow && change(connect(flow, { node: c.source, port: port(c.source, 'outputs') }, { node: c.target, port: port(c.target, 'inputs') }))}
          ondelete={remove}
          ondropnode={(typeId, at) => catalog[typeId] && add(catalog[typeId], at)}
        />
      {/if}
      {#if selectedNode && !proposal && !listView}
        <div class="selection" role="toolbar" aria-label="Selected step">
          <span class="sel-name">{selectedNode.name}</span>
          <Button icon="swap" size="s" variant="ghost" onclick={() => (palette = 'replace')}>Replace with…</Button>
          <IconButton icon="trash" label="Delete step" onclick={() => remove([selectedNode.id])} />
        </div>
      {/if}
      {#if asking}
        <div class="ask-slot">
          <AskBar
            pending={askPending}
            summary={proposal?.summary ?? null}
            onask={ask}
            onaccept={accept}
            ondiscard={() => (proposal = null)}
            onclose={() => (asking = false)}
          />
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .editor { display: flex; flex-direction: column; height: 100vh; min-height: 0; }
  .body { flex: 1 1 auto; min-height: 0; display: flex; }
  .stage { position: relative; flex: 1 1 auto; min-width: 0; overflow: hidden; }
  .stage.list { overflow: auto; }
  .list-wrap { max-width: 520px; margin: 0 auto; padding: 24px 16px 140px; }
  .state { display: flex; align-items: center; gap: 8px; padding: 32px; color: var(--text-3); }
  .selection {
    position: absolute; top: 16px; left: 50%; transform: translateX(-50%); z-index: 5; display: flex; align-items: center; gap: 4px;
    padding: 4px 4px 4px 14px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-md);
  }
  .sel-name { font-size: 13px; font-weight: 600; margin-right: 6px; max-width: 220px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .ask-slot { position: absolute; left: 50%; bottom: 16px; transform: translateX(-50%); z-index: 6; }
  @media (max-width: 767px) { .ask-slot { bottom: 80px; } }
</style>
