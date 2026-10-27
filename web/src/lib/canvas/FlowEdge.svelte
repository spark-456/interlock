<script lang="ts">
  // Edge: cubic bezier (DESIGN.md). ready = solid grey, queued = dashed light grey,
  // active = teal base + animated primary dash, failed = red. Vertical edges are dotted links.
  // A run's item count sits on the curve's midpoint.
  import { EdgeLabel, type Edge, type EdgeProps } from '@xyflow/svelte';
  import { bezier, verticalBezier } from './geometry';
  import type { EdgeView } from './types';

  let { sourceX, sourceY, targetX, targetY, data }: EdgeProps<Edge<{ view: EdgeView }, 'flow'>> = $props();
  const view = $derived(data?.view);
  const d = $derived(view?.vertical ? verticalBezier(sourceX, sourceY, targetX, targetY) : bezier(sourceX, sourceY, targetX, targetY));
  const state = $derived(view?.state ?? 'ready');
</script>

{#if state === 'active'}
  <path {d} class="base" fill="none" />
  <path {d} class="flow" fill="none" data-state="active" />
{:else}
  <path {d} class="edge {state}" class:dotted={view?.vertical} fill="none" data-state={state} />
{/if}
{#if view?.count !== undefined}
  <EdgeLabel x={(sourceX + targetX) / 2} y={(sourceY + targetY) / 2} transparent>
    <span class="count" data-testid="edge-count">{view.count} {view.count === 1 ? 'item' : 'items'}</span>
  </EdgeLabel>
{/if}

<style>
  path { stroke-linecap: round; }
  .edge { stroke: var(--edge); stroke-width: 1.5; }
  .edge.queued { stroke: var(--edge-queued); stroke-dasharray: 4 6; opacity: .8; }
  .edge.failed { stroke: var(--error); }
  .edge.dotted { stroke-dasharray: 2 5; }
  .base { stroke: var(--primary-line); stroke-width: 2; }
  .flow { stroke: var(--primary); stroke-width: 2; stroke-dasharray: 6 8; animation: dash var(--dur-edge-dash) linear infinite; }
  @keyframes dash { from { stroke-dashoffset: 28; } to { stroke-dashoffset: 0; } }
  .count {
    display: inline-block; padding: 1px 6px; border-radius: 999px; background: var(--surface); border: 1px solid var(--border);
    font: 500 11px/16px var(--font-mono); color: var(--text-2);
  }
  @media (prefers-reduced-motion: reduce) { .flow { animation: none; } }
</style>
