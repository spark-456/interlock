<script lang="ts">
  // Edge: cubic bezier (DESIGN.md). ready = solid grey, queued = dashed light grey,
  // active = teal base + animated primary dash, failed = red. Vertical edges are dotted links.
  import type { Edge, EdgeProps } from '@xyflow/svelte';
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

<style>
  path { stroke-linecap: round; }
  .edge { stroke: var(--edge); stroke-width: 1.5; }
  .edge.queued { stroke: var(--edge-queued); stroke-dasharray: 4 6; opacity: .8; }
  .edge.failed { stroke: var(--error); }
  .edge.dotted { stroke-dasharray: 2 5; }
  .base { stroke: var(--primary-line); stroke-width: 2; }
  .flow { stroke: var(--primary); stroke-width: 2; stroke-dasharray: 6 8; animation: dash var(--dur-edge-dash) linear infinite; }
  @keyframes dash { from { stroke-dashoffset: 28; } to { stroke-dashoffset: 0; } }
  @media (prefers-reduced-motion: reduce) { .flow { animation: none; } }
</style>
