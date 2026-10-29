<script lang="ts">
  // Run view: the flow read-only on the canvas, animated from the run stream, beside the run's
  // summary, its approval, the clips it made and every step.
  import { onDestroy } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';
  import { page } from '$app/state';
  import { api, runEvents, type Approval, type Flow, type NodeManifest } from '$lib/api';
  import NodeCanvas from '$lib/canvas/NodeCanvas.svelte';
  import NodeList from '$lib/canvas/NodeList.svelte';
  import { columnLabels } from '$lib/canvas/geometry';
  import { idleRun, reduceRun } from '$lib/canvas/runState';
  import { flowCards, flowEdges, groupViews } from '$lib/canvas/views';
  import { clock, usd } from '$lib/design/format';
  import Banner from '$lib/design/Banner.svelte';
  import StatusPill, { type PillTone } from '$lib/design/StatusPill.svelte';
  import TopBar from '$lib/design/TopBar.svelte';
  import FriendlyError from '$lib/errors/FriendlyError.svelte';
  import ApprovalCard from '../../../../approvals/ApprovalCard.svelte';
  import { emptyView, reduceView, type StepStatus } from './run';

  const flowId = page.params.id ?? '';
  const runId = page.params.run ?? '';
  const speed = Number(page.url.searchParams.get('speed') ?? import.meta.env.VITE_FIXTURE_SPEED ?? 1) || 1;
  const mobile = new MediaQuery('max-width: 767px');

  let flow = $state.raw<Flow | null>(null);
  let manifests = $state.raw<NodeManifest[]>([]);
  let approval = $state.raw<Approval | null>(null);
  let run = $state.raw(idleRun());
  let view = $state.raw(emptyView());

  api.flow(flowId).then((f) => (flow = f), () => {});
  api.nodes().then((c) => (manifests = c.nodes), () => {});
  const sub = runEvents(runId, (e) => {
    run = reduceRun(run, e);
    view = reduceView(view, e);
  }, { speed });
  onDestroy(() => sub.close());

  // The approval this run is (or was) waiting on, for its card.
  $effect(() => {
    const id = view.approval?.id;
    if (id && approval?.id !== id) api.approvals().then((list) => (approval = list.find((a) => a.id === id) ?? null), () => {});
  });

  const catalog = $derived(Object.fromEntries(manifests.map((m) => [m.typeId, m])));
  const cards = $derived(flow ? flowCards(flow, catalog, run) : []);
  const edges = $derived(flow ? flowEdges(flow, run) : []);
  const labels = $derived(flow ? columnLabels(cards, flow.groups) : []);
  const groups = $derived(flow ? groupViews(flow.groups, cards) : []);
  const name = (nodeId: string) => flow?.nodes.find((n) => n.id === nodeId)?.name ?? nodeId;
  // The gallery shows the last step that made media: the finished clips.
  const clips = $derived(view.clips.filter((c) => c.nodeId === view.clips.at(-1)?.nodeId));
  const decision = $derived(view.approval?.decision);

  const RUN: Record<typeof view.status, [PillTone, string]> = {
    running: ['building', 'Running'],
    succeeded: ['ready', 'Succeeded'],
    failed: ['failed', 'Failed'],
    capped: ['warning', 'Cap reached']
  };
  const STEP: Record<StepStatus, [PillTone, string]> = {
    running: ['building', 'Running'],
    waiting: ['warning', 'Waiting'],
    succeeded: ['ready', 'Done'],
    failed: ['failed', 'Failed']
  };
</script>

<svelte:head><title>Run · {flow?.name ?? 'Interlock'}</title></svelte:head>

<div class="screen">
  <TopBar title="Run" crumb={{ href: `/flows/${flowId}`, label: flow?.name ?? 'Flow' }} back="/flows/{flowId}">
    {#snippet status()}<StatusPill tone={RUN[view.status][0]} label={RUN[view.status][1]} size={26} />{/snippet}
  </TopBar>
  <div class="body">
    <div class="stage">
      {#if flow && mobile.current}
        <div class="list-wrap"><NodeList {groups} onselect={() => {}} /></div>
      {:else if flow}
        <NodeCanvas {cards} {edges} {labels} />
      {/if}
    </div>
    <aside aria-label="Run details">
      <dl class="summary">
        <div><dt>Status</dt><dd>{RUN[view.status][1]}</dd></div>
        <div><dt>Started by</dt><dd>{view.trigger ?? '—'}</dd></div>
        <div><dt>Duration</dt><dd class="mono">{view.ms !== undefined ? clock(view.ms) : '—'}</dd></div>
        <div><dt>Cost</dt><dd class="mono">{view.costUsd !== undefined ? usd(view.costUsd) : '—'}</dd></div>
      </dl>
      {#if view.status === 'capped'}
        <Banner kind="cap">{view.error}. The run stopped instead of spending more.</Banner>
      {:else if view.error}<FriendlyError error={view.error} />{/if}

      {#if approval && approval.status === 'pending' && !decision}
        <ApprovalCard {approval} ondecided={() => (approval = approval && { ...approval, status: 'approved' })} />
      {:else if view.approval && decision}
        <p class="decided">{decision === 'approve' ? 'Approved' : 'Rejected'}{view.approval.by ? ` by ${view.approval.by}` : ''} at “{name(view.approval.nodeId)}”.</p>
      {/if}

      {#if clips.length}
        <section aria-label="Clips">
          <h2>Clips <span class="muted">{name(clips[0]!.nodeId)}</span></h2>
          <div class="clips">
            {#each clips as c (c.key)}
              <figure>
                {#if c.video}<video src={c.url} controls muted preload="metadata" aria-label={c.title}></video>
                {:else}<img src={c.url} alt={c.title} />{/if}
                <figcaption>{c.title}</figcaption>
              </figure>
            {/each}
          </div>
        </section>
      {/if}

      <section aria-label="Steps">
        <h2>Steps</h2>
        <ol class="steps">
          {#each view.steps as s (s.nodeId)}
            <li>
              <div class="row">
                <span class="name">{name(s.nodeId)}</span>
                <StatusPill tone={STEP[s.status][0]} label={STEP[s.status][1]} />
              </div>
              {#if s.status === 'succeeded'}
                <span class="meta mono">{s.count} {s.count === 1 ? 'item' : 'items'} · {clock(s.ms ?? 0)}{s.costUsd ? ` · ${usd(s.costUsd)}` : ''}</span>
              {:else if s.error && s.error !== view.error}
                <FriendlyError error={s.error} />
              {/if}
            </li>
          {:else}
            <li class="muted">Waiting for the run to start…</li>
          {/each}
        </ol>
      </section>
    </aside>
  </div>
</div>

<style>
  .screen { display: flex; flex-direction: column; height: 100vh; min-height: 0; }
  .body { flex: 1 1 auto; min-height: 0; display: flex; }
  .stage { position: relative; flex: 1 1 auto; min-width: 0; overflow: hidden; }
  .list-wrap { max-width: 520px; margin: 0 auto; padding: var(--s-6) var(--s-4); }
  aside {
    width: 380px; flex: none; overflow-y: auto; padding: var(--s-5); display: flex; flex-direction: column; gap: var(--s-5);
    background: var(--surface); border-left: 1px solid var(--border);
  }
  .summary { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-3); margin: 0; }
  dt { color: var(--text-3); font: var(--t-small); }
  dd { margin: var(--s-1) 0 0; color: var(--text); }
  .mono { font-family: var(--font-mono); }
  h2 { font: var(--t-h3); margin: 0 0 var(--s-3); }
  .muted { color: var(--text-3); font: var(--t-small); }
  .decided { margin: 0; padding: var(--s-3) var(--s-4); border-radius: var(--r-card); background: var(--success-soft); color: var(--success-text); }
  .clips { display: grid; grid-template-columns: repeat(auto-fill, minmax(100px, 1fr)); gap: var(--s-3); }
  figure { margin: 0; display: flex; flex-direction: column; gap: var(--s-1); }
  video, img { width: 100%; aspect-ratio: 9 / 16; object-fit: cover; border-radius: var(--r-card); background: var(--surface-4); }
  figcaption { font: var(--t-small); color: var(--text-2); }
  .steps { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; }
  .steps li { padding: var(--s-2) 0; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: var(--s-1); }
  .row { display: flex; justify-content: space-between; align-items: center; gap: var(--s-2); }
  .name { font-weight: 600; }
  .meta { font: var(--t-small); color: var(--text-3); }
  @media (max-width: 767px) {
    .body { flex-direction: column; overflow-y: auto; }
    .stage { flex: none; overflow: visible; }
    aside { width: auto; border-left: 0; border-top: 1px solid var(--border); overflow: visible; }
  }
</style>
