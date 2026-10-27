<script lang="ts">
  // S3 / M2 Building: everything on screen comes from the build SSE stream (PRD §8.4).
  import { onDestroy } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, buildEvents, type BuildEvent } from '$lib/api';
  import {
    STAGES,
    currentStageIndex,
    initialBuildState,
    progress,
    reduce,
    testsRun,
    timeLeft,
    type BuildState
  } from '$lib/canvas/buildState';
  import NodeCanvas from '$lib/canvas/NodeCanvas.svelte';
  import NodeList from '$lib/canvas/NodeList.svelte';
  import { cardViews, edgeViews, groupViews, labelViews, type Catalog } from '$lib/canvas/views';
  import Button from '$lib/design/Button.svelte';
  import Spinner from '$lib/design/Spinner.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import TopBar from '$lib/design/TopBar.svelte';
  import { clock, usd } from '$lib/design/format';
  import BuildLog from './BuildLog.svelte';
  import FixCard from './FixCard.svelte';
  import Stepper from './Stepper.svelte';

  const buildId = page.params.id ?? '';
  // Fixture replay controls (ignored by the live server): ?speed=4 and ?at=88 (freeze at 1:28).
  const query = page.url.searchParams;
  const speed = Number(query.get('speed') ?? import.meta.env.VITE_FIXTURE_SPEED ?? 1) || 1;
  const until = query.has('at') ? Number(query.get('at')) * 1000 : undefined;
  let paused = $state(false);

  let build = $state.raw<BuildState>(initialBuildState());
  let catalog = $state.raw<Catalog>({});
  let listView = $state(false);
  const mobile = new MediaQuery('max-width: 767px');
  let selected = $state<string | null>(null);
  let arrivedAt = $state(performance.now());
  let tick = $state(performance.now());

  api.nodes().then((c) => (catalog = Object.fromEntries(c.nodes.map((n) => [n.typeId, n])))).catch(() => {});

  const sub = buildEvents(buildId, (e: BuildEvent) => {
    build = reduce(build, e);
    arrivedAt = performance.now();
  }, { speed, until, onEnd: () => (paused = true) });
  const timer = setInterval(() => (tick = performance.now()), 250);
  onDestroy(() => {
    sub.close();
    clearInterval(timer);
  });

  // Elapsed snaps to event time and ticks between events (capped so it never runs ahead).
  const now = $derived(
    build.status === 'building' && build.t0 !== null && !paused ? build.elapsed + Math.min((tick - arrivedAt) * speed, 2500) : build.elapsed
  );
  const cards = $derived(cardViews(build, catalog));
  const edges = $derived(edgeViews(build));
  const labels = $derived(labelViews(build, cards));
  const groups = $derived(groupViews(build, cards));
  const stepIndex = $derived(currentStageIndex(build));
  const stage = $derived(STAGES[stepIndex]!);
  const pct = $derived(progress(build));
  const left = $derived(timeLeft(build));
  const run = $derived(testsRun(build));
  const name = $derived(build.flow.name ?? 'your flow');
  const fixing = $derived(build.stages['test']?.status === 'active');

  const stepLine = $derived(
    build.status === 'ready'
      ? 'All steps done'
      : `Step ${stepIndex + 1} of ${STAGES.length} · ${stage.key === 'test' ? 'Testing' : stage.label}`
  );
  const mobileStep = $derived(
    stage.key === 'test' && run && build.status !== 'ready'
      ? `Step ${stepIndex + 1} of ${STAGES.length} · ${run.run} of ${run.total} tests run`
      : stepLine
  );

  function cancel() {
    void goto('/flows');
  }
</script>

<svelte:head><title>Building {name} · Interlock</title></svelte:head>

<div class="building">
  <div class="main">
    <TopBar
      title={build.status === 'ready' ? name : `Building ${name}`}
      crumb={{ href: '/flows', label: 'Flows' }}
      subtitle={build.status === 'ready' ? 'Ready' : 'Building your flow'}
      back="/flows"
      onmenu={() => (listView = !listView)}
    >
      {#snippet status()}
        {#if build.status === 'ready'}
          <StatusPill tone="ready" label="Ready · {clock(build.elapsed)}" size={26} />
        {:else if build.status === 'failed'}
          <StatusPill tone="failed" label="Failed" size={26} />
        {:else}
          <span class="live-pill" data-testid="build-timer"><Spinner size={12} />Building · <span class="mono">{clock(now)}</span></span>
        {/if}
      {/snippet}
      {#snippet actions()}
        <Button icon="eye" variant="secondary" size="s" aria-pressed={listView} onclick={() => (listView = !listView)}>
          {listView ? 'Canvas' : 'Inspect'}
        </Button>
        {#if build.status === 'ready' && build.flowId}
          <Button variant="primary" size="s" href="/flows/{build.flowId}">Open editor</Button>
        {:else}
          <Button variant="secondary" size="s" onclick={cancel}>Cancel</Button>
        {/if}
      {/snippet}
    </TopBar>

    <!-- Mobile progress card (M2) -->
    <div class="m-progress">
      <div class="m-card">
        <div class="m-row">
          {#if build.status === 'ready'}<StatusPill tone="ready" label="Ready" size={26} />
          {:else}<span class="live-pill"><Spinner size={12} />{stage.key === 'test' ? 'Testing' : stage.label}</span>{/if}
          <span class="m-clock mono">{clock(now)}</span>
        </div>
        <div class="segments" role="progressbar" aria-label="Build progress, step {stepIndex + 1} of {STAGES.length}" aria-valuemin={0} aria-valuemax={STAGES.length} aria-valuenow={stepIndex + 1}>
          {#each STAGES as s, i (s.key)}
            {@const st = build.stages[s.key]?.status}
            {@const frac = st === 'done' || build.status === 'ready' ? 100 : st === 'active' ? Math.round((pct * STAGES.length - i) * 100) : 0}
            <span class="seg" style:--fill="{Math.max(0, Math.min(100, frac))}%"></span>
          {/each}
        </div>
        <div class="seg-labels">
          {#each STAGES as s, i (s.key)}<span class:current={i === stepIndex && build.status !== 'ready'}>{s.short}</span>{/each}
        </div>
        <div class="m-foot"><span>{mobileStep}</span>{#if left}<span>About {Math.round(left / 1000)} s left</span>{/if}</div>
      </div>
    </div>

    <div class="stage" class:list={listView}>
      {#if listView || mobile.current}
        <div class="list-wrap"><NodeList {groups} onselect={(id) => (selected = id)} /></div>
      {:else}
        <NodeCanvas {cards} {edges} {labels} bind:selected />
      {/if}
      <div class="fix-slot"><FixCard fixes={build.fixes} {fixing} failing={build.tests?.failed ?? 0} /></div>
    </div>

    <!-- Mobile log + actions (M2) -->
    <div class="m-log">
      <div class="m-head"><span>Build log</span><span class="muted">Latest</span></div>
      <div class="m-well"><BuildLog lines={build.log} limit={4} /></div>
      <div class="m-actions">
        <div class="stat"><div class="label">Cost so far</div><div class="value mono">{usd(build.costUsd)}</div></div>
        {#if build.status === 'ready' && build.flowId}
          <Button variant="primary" href="/flows/{build.flowId}">Open editor</Button>
        {:else}
          <Button variant="secondary" onclick={cancel}>Cancel build</Button>
        {/if}
      </div>
    </div>
  </div>

  <aside class="panel" aria-label="Build log">
    <div class="p-head">
      <div class="p-title"><h2>Build log</h2><span class="muted">{build.status === 'building' ? 'Live' : build.status === 'ready' ? 'Done' : 'Stopped'}</span></div>
      <div class="bar"><div style:width="{Math.round(pct * 100)}%"></div></div>
      <div class="p-sub"><span>{stepLine}</span>{#if left}<span>About {Math.round(left / 1000)} s left</span>{/if}</div>
    </div>
    <Stepper {build} {now} />
    <div class="well"><BuildLog lines={build.log} /></div>
    <div class="stats">
      <div class="stat"><div class="label">Elapsed</div><div class="value mono">{clock(now)}</div></div>
      <div class="stat"><div class="label">Cost so far</div><div class="value mono" data-testid="build-cost">{usd(build.costUsd)}</div></div>
    </div>
  </aside>
</div>

<style>
  .building { display: flex; height: 100vh; min-height: 0; }
  .main { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; }
  .mono { font-family: var(--font-mono); }
  .muted { font-size: 12px; color: var(--text-3); font-weight: 400; }
  .live-pill {
    display: inline-flex; align-items: center; gap: 6px; height: 26px; padding: 0 10px; border-radius: var(--r-pill);
    background: var(--pill-building-bg); color: var(--pill-building-fg); font-size: 12px; font-weight: 600; white-space: nowrap;
  }
  .stage { position: relative; flex: 1 1 auto; min-height: 0; overflow: hidden; }
  .stage.list { overflow: auto; }
  .list-wrap { max-width: 520px; margin: 0 auto; padding: 24px 16px 140px; }
  .fix-slot { position: absolute; left: 236px; bottom: 16px; width: 476px; z-index: 5; }

  .panel {
    flex: 0 0 var(--inspector); width: var(--inspector); height: 100vh; display: flex; flex-direction: column;
    background: var(--surface); border-left: 1px solid var(--border);
  }
  .p-head { padding: 20px 24px 16px; border-bottom: 1px solid var(--border); }
  .p-title { display: flex; align-items: center; justify-content: space-between; }
  h2 { font-size: 18px; line-height: 26px; font-weight: 700; }
  .bar { height: 8px; border-radius: var(--r-pill); background: var(--surface-4); margin-top: 14px; overflow: hidden; }
  .bar div { height: 8px; border-radius: var(--r-pill); background: linear-gradient(90deg, var(--primary), var(--primary-bright)); transition: width var(--dur-panel) ease; }
  .p-sub { display: flex; justify-content: space-between; margin-top: 8px; font-size: 12px; color: var(--text-2); }
  .well {
    flex: 0 1 auto; min-height: 0; margin: 8px 16px 0; padding: 12px; border-radius: var(--r-card); background: var(--surface-3);
    border: 1px solid var(--surface-2); overflow: hidden; display: flex; flex-direction: column; justify-content: flex-end;
  }
  .stats { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; padding: 16px 24px 20px; margin-top: auto; }
  .stat { padding: 10px 12px; border: 1px solid var(--border); border-radius: var(--r-tile); background: var(--surface); }
  .label { font-size: 12px; color: var(--text-3); }
  .value { font-size: 16px; font-weight: 600; margin-top: 2px; }

  .m-progress, .m-log { display: none; }

  @media (max-width: 1100px) { .panel { flex-basis: var(--rail); width: var(--rail); } .fix-slot { left: 16px; width: min(476px, calc(100% - 32px)); bottom: 80px; } }

  @media (max-width: 767px) {
    .building { height: auto; min-height: 100%; }
    .panel { display: none; }
    .stage { overflow: visible; flex: none; }
    .list-wrap { padding: 8px 16px 0; max-width: none; }
    .fix-slot { position: static; width: auto; padding: 16px 16px 0; }
    .m-progress { display: block; padding: 16px 16px 0; }
    .m-card { padding: 16px; border-radius: var(--r-panel); background: var(--card-gradient); border: 1px solid var(--border); box-shadow: var(--shadow-pop); }
    .m-row { display: flex; align-items: center; justify-content: space-between; }
    .m-clock { font-size: 20px; font-weight: 600; }
    .segments { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: 4px; margin-top: 14px; }
    .seg { height: 6px; border-radius: 3px; background: linear-gradient(90deg, var(--primary) var(--fill), var(--surface-4) var(--fill)); }
    .seg-labels { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: 4px; margin-top: 6px; font-size: 12px; line-height: 16px; color: var(--text-4); text-align: center; }
    .seg-labels .current { color: var(--primary); font-weight: 700; }
    .m-foot { display: flex; justify-content: space-between; margin-top: 12px; padding-top: 12px; border-top: 1px solid var(--surface-2); font-size: 13px; color: var(--text-2); }
    .m-log { display: block; padding: 20px 16px 24px; }
    .m-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; font-size: 15px; font-weight: 700; }
    .m-well { padding: 12px; border-radius: var(--r-card); background: var(--surface); border: 1px solid var(--border); }
    .m-actions { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; margin-top: 10px; }
    .m-actions :global(.btn) { height: 100%; min-height: 52px; border-radius: var(--r-tile); }
  }
</style>
