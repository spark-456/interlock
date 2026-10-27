<script lang="ts">
  // One flow on the list (design S7): a canvas thumbnail, status, last run, a 14-day run sparkline and its apps.
  import { api } from '$lib/api';
  import type { Flow, FlowSummary, RunSummary } from '$lib/api/types';
  import { toneFor, toneVars } from '$lib/canvas/categories';
  import AppTile from '$lib/design/AppTile.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import { ago, perDay, sparkPoints, thumbnail } from './flows';

  let { flow, appNames, now }: { flow: FlowSummary; appNames: Record<string, string>; now: Date } = $props();

  let full = $state.raw<Flow | null>(null);
  let runs = $state.raw<RunSummary[]>([]);
  $effect(() => {
    api.flow(flow.id).then((f) => (full = f), () => {});
    api.runs(flow.id).then((r) => (runs = r), () => {});
  });

  const thumb = $derived(full && full.nodes.length ? thumbnail(full, 300, 112) : null);
  const days = $derived(perDay(runs, 14, now));
  const total = $derived(days.reduce((a, b) => a + b, 0));
  const app = $derived(flow.kind === 'app');
  const last = $derived(flow.lastRun);
</script>

<article class="card">
  <div class="thumb" aria-hidden="true">
    {#if thumb}
      <svg viewBox="0 0 300 112" preserveAspectRatio="xMidYMid meet">
        {#each thumb.edges as e (e.id)}
          <path d="M{e.x1},{e.y1} C{(e.x1 + e.x2) / 2},{e.y1} {(e.x1 + e.x2) / 2},{e.y2} {e.x2},{e.y2}" />
        {/each}
        {#each thumb.nodes as n (n.id)}
          <rect x={n.x} y={n.y} width="28" height="10" rx="3" style:fill={toneVars(toneFor(n.type)).tile} style:stroke={toneVars(toneFor(n.type)).edge} />
          <rect x={n.x + 3} y={n.y + 3} width="4" height="4" rx="1" style:fill={toneVars(toneFor(n.type)).icon} />
        {/each}
      </svg>
    {/if}
  </div>
  <div class="head">
    <a class="name" href="/flows/{flow.id}">{flow.name}</a>
    <StatusPill tone={flow.active ? 'live' : 'waiting'} label={flow.active ? 'Live' : 'Off'} />
  </div>
  <p class="kind">{app ? 'AI app' : 'Automation'} · v{flow.version}</p>
  <dl>
    <div>
      <dt>Last run</dt>
      <dd>
        {#if last}<a href="/flows/{flow.id}/runs/{last.id}">{last.status === 'failed' ? 'Failed' : 'Ran'} {ago(last.startedAt, now)}</a>{:else}Never{/if}
      </dd>
    </div>
    <div>
      <dt>Runs, 14 days</dt>
      <dd class="runs">
        <span class="mono">{total}</span>
        <svg class="spark" viewBox="0 0 64 18" role="img" aria-label="{total} runs in the last 14 days">
          <polyline points={sparkPoints(days, 64, 18)} />
        </svg>
      </dd>
    </div>
    <div>
      <dt>Apps</dt>
      <dd class="apps">
        {#each flow.apps.slice(0, 4) as a (a)}<span title={appNames[a] ?? a}><AppTile name={appNames[a] ?? a} size={20} /></span>{/each}
        {#if flow.apps.length > 4}<span class="more">+{flow.apps.length - 4}</span>{/if}
        {#if !flow.apps.length}<span class="none">—</span>{/if}
      </dd>
    </div>
  </dl>
  <footer>
    <span>Updated {ago(flow.updatedAt, now)}</span>
    <a href={app ? `/flows/${flow.id}/publish` : `/flows/${flow.id}`}>{app ? 'Connect' : 'Open'} <Icon name="arrow-right" size={14} /></a>
  </footer>
</article>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-3); padding: var(--s-3) var(--s-4) var(--s-4);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
    transition: box-shadow var(--dur-pill) ease;
  }
  .card:hover { box-shadow: var(--shadow-pop-hover); }
  .thumb {
    height: 112px; border-radius: var(--r-card); background-color: var(--surface-3);
    background-image: radial-gradient(var(--dot-grid) 1px, transparent 1px); background-size: 12px 12px;
  }
  .thumb svg { width: 100%; height: 100%; }
  .thumb path { fill: none; stroke: var(--edge); stroke-width: 1; }
  .head { display: flex; align-items: center; justify-content: space-between; gap: var(--s-2); }
  .name { font: var(--t-h3); color: var(--text); text-decoration: none; }
  .name:hover { text-decoration: underline; }
  .kind { margin: calc(-1 * var(--s-2)) 0 0; color: var(--text-3); font: var(--t-small); }
  dl { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--s-2); margin: 0; }
  dt { color: var(--text-3); font: var(--t-small); }
  dd { margin: var(--s-1) 0 0; display: flex; align-items: center; gap: var(--s-1); min-height: 20px; font: var(--t-small); color: var(--text); }
  dd a { color: inherit; }
  .mono { font-family: var(--font-mono); }
  .spark { width: 64px; height: 18px; }
  .spark polyline { fill: none; stroke: var(--primary); stroke-width: 1.5; stroke-linejoin: round; }
  .apps { flex-wrap: wrap; }
  .more, .none { color: var(--text-3); }
  footer {
    display: flex; justify-content: space-between; align-items: center; padding-top: var(--s-3);
    border-top: 1px solid var(--border); color: var(--text-3); font: var(--t-small);
  }
  footer a { display: inline-flex; align-items: center; gap: var(--s-1); color: var(--primary); font-weight: 600; text-decoration: none; }
</style>
