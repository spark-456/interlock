<script lang="ts">
  // NodeCanvas: Svelte Flow does drag, pan and zoom; we supply the design's NodeCard, edges,
  // column labels, toolbar, minimap and 20 px dot grid. Positions the user drags are kept when
  // new data arrives (live builds and runs).
  import {
    Background,
    BackgroundVariant,
    MiniMap,
    Panel,
    SvelteFlow,
    ViewportPortal,
    type Edge,
    type Node,
    type Viewport
  } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/base.css';
  import { untrack } from 'svelte';
  import CanvasToolbar from './CanvasToolbar.svelte';
  import FlowEdge from './FlowEdge.svelte';
  import NodeCard from './NodeCard.svelte';
  import { toneVars } from './categories';
  import { MAX_ZOOM, MIN_ZOOM, NODE_WIDTH, nodeHeight, type ColumnLabel } from './geometry';
  import type { CardView, EdgeView } from './types';

  type CardNode = Node<{ card: CardView }, 'card'>;
  type FlowEdgeT = Edge<{ view: EdgeView }, 'flow'>;

  let {
    cards,
    edges: edgeViews,
    labels = [],
    initialViewport = { x: 24, y: 132, zoom: 0.8 },
    selected = $bindable<string | null>(null),
    onlist,
    showToolbar = true
  }: {
    cards: CardView[];
    edges: EdgeView[];
    labels?: ColumnLabel[];
    initialViewport?: Viewport;
    selected?: string | null;
    onlist?: () => void;
    showToolbar?: boolean;
  } = $props();

  const nodeTypes = { card: NodeCard };
  const edgeTypes = { flow: FlowEdge };

  let nodes = $state.raw<CardNode[]>([]);
  let edges = $state.raw<FlowEdgeT[]>([]);
  let minimap = $state(true);

  // Merge incoming views into Svelte Flow nodes, keeping positions the user dragged.
  $effect(() => {
    const views = cards;
    const current = untrack(() => nodes);
    const byId = new Map(current.map((n) => [n.id, n]));
    nodes = views.map((card) => {
      const prev = byId.get(card.id);
      return {
        id: card.id,
        type: 'card',
        position: prev?.position ?? { ...card.position },
        data: { card },
        width: NODE_WIDTH,
        height: nodeHeight(card.stats.length),
        selected: prev?.selected ?? false
      };
    });
  });

  $effect(() => {
    edges = edgeViews.map((view) => ({
      id: view.id,
      type: 'flow',
      source: view.source,
      target: view.target,
      sourceHandle: view.vertical ? 'bottom' : null,
      targetHandle: view.vertical ? 'top' : null,
      data: { view },
      selectable: false
    }));
  });

  const miniColor = (n: Node) => {
    const card = (n.data as { card?: CardView }).card;
    if (n.id === selected) return 'var(--primary)';
    return card?.status === 'waiting' ? 'var(--border)' : 'var(--minimap-node)';
  };
</script>

<div class="canvas" data-testid="node-canvas">
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    {edgeTypes}
    {initialViewport}
    minZoom={MIN_ZOOM}
    maxZoom={MAX_ZOOM}
    nodesConnectable={false}
    zoomOnScroll={false}
    panOnScroll={true}
    zoomOnPinch={true}
    proOptions={{ hideAttribution: true }}
    onnodeclick={({ node }) => (selected = node.id)}
    onpaneclick={() => (selected = null)}
  >
    <Background variant={BackgroundVariant.Dots} gap={20} size={2.2} bgColor="var(--bg)" patternColor="var(--dot-grid)" />
    <ViewportPortal target="back">
      {#each labels as l (l.text)}
        <div class="col-label" style:transform="translate({l.x}px, {l.y}px)">
          <span class="sq" style:background={toneVars(l.tone as never).icon}></span>{l.text}
        </div>
      {/each}
    </ViewportPortal>
    {#if showToolbar}
      <Panel position="bottom-left"><CanvasToolbar bind:minimap {onlist} /></Panel>
      {#if minimap}
        <MiniMap
          width={180}
          height={120}
          position="bottom-right"
          class="il-minimap"
          nodeColor={miniColor}
          nodeBorderRadius={2}
          maskColor="var(--minimap-mask)"
          maskStrokeColor="var(--primary)"
          maskStrokeWidth={1.5}
          ariaLabel="Minimap"
        />
      {/if}
    {/if}
  </SvelteFlow>
</div>

<style>
  .canvas { position: absolute; inset: 0; touch-action: none; }
  .canvas :global(.svelte-flow) { --xy-background-color: var(--bg); font-family: var(--font-sans); }
  .canvas :global(.svelte-flow__node) { border: 0; padding: 0; background: transparent; box-shadow: none; }
  .canvas :global(.svelte-flow__node:focus-visible) { outline: none; }
  .canvas :global(.svelte-flow__node:focus-visible .card) { box-shadow: var(--shadow-pop-selected); }
  .canvas :global(.svelte-flow__panel) { margin: 16px; }
  .canvas :global(.il-minimap) {
    margin: 16px; border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-md);
    background: var(--surface); overflow: hidden;
  }
  .col-label {
    position: absolute; left: 0; top: 0; width: 264px; display: flex; align-items: center; gap: 8px;
    font-size: 12px; line-height: 16px; font-weight: 700; letter-spacing: .06em; text-transform: uppercase; color: var(--text-4);
    pointer-events: none;
  }
  .sq { width: 6px; height: 6px; border-radius: 2px; }
</style>
