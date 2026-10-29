<script lang="ts">
  // NodeCanvas: Svelte Flow does drag, pan and zoom; we supply the design's NodeCard, edges,
  // column labels, toolbar, minimap and 20 px dot grid. Read-only screens keep positions the user
  // drags when new data arrives (live builds); in the editor the flow owns positions, and drags,
  // connections, deletions and palette drops are reported to the page.
  import {
    Background,
    BackgroundVariant,
    MiniMap,
    Panel,
    SvelteFlow,
    ViewportPortal,
    type Connection,
    type Edge,
    type Node,
    type Viewport
  } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/base.css';
  import { untrack } from 'svelte';
  import CanvasToolbar, { type EditTools } from './CanvasToolbar.svelte';
  import FlowEdge from './FlowEdge.svelte';
  import Focus from './Focus.svelte';
  import NodeCard from './NodeCard.svelte';
  import { toneVars } from './categories';
  import { MAX_ZOOM, MIN_ZOOM, NODE_WIDTH, nodeHeight, type ColumnLabel } from './geometry';
  import type { CardView, EdgeView } from './types';
  import { PALETTE_MIME } from './palette';

  type CardNode = Node<{ card: CardView }, 'card'>;
  type FlowEdgeT = Edge<{ view: EdgeView }, 'flow'>;

  let {
    cards,
    edges: edgeViews,
    labels = [],
    selected = $bindable<string | null>(null),
    onlist,
    focus = [],
    editable = false,
    edit,
    onmove,
    onconnect,
    ondelete,
    ondropnode
  }: {
    cards: CardView[];
    edges: EdgeView[];
    labels?: ColumnLabel[];
    selected?: string | null;
    onlist?: () => void;
    /** Nodes to bring into view when this list changes. */
    focus?: string[];
    editable?: boolean;
    edit?: EditTools;
    onmove?: (moves: Record<string, { x: number; y: number }>) => void;
    onconnect?: (c: Connection) => void;
    ondelete?: (nodeIds: string[], edgeIds: string[]) => void;
    /** A palette item dropped on the canvas, at its flow position. */
    ondropnode?: (typeId: string, at: { x: number; y: number }) => void;
  } = $props();

  const nodeTypes = { card: NodeCard };
  const edgeTypes = { flow: FlowEdge };

  let nodes = $state.raw<CardNode[]>([]);
  let edges = $state.raw<FlowEdgeT[]>([]);
  let minimap = $state(true);
  let viewport = $state<Viewport>({ x: 24, y: 132, zoom: 0.8 });
  let el: HTMLDivElement;

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
        position: editable ? { ...card.position } : (prev?.position ?? { ...card.position }),
        data: { card },
        width: NODE_WIDTH,
        height: nodeHeight(card.stats.length),
        selected: prev?.selected ?? false,
        draggable: card.diff !== 'removed',
        deletable: editable && !card.diff
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
      selectable: editable && !view.diff,
      deletable: editable && !view.diff
    }));
  });

  const miniColor = (n: Node) => {
    const card = (n.data as { card?: CardView }).card;
    if (n.id === selected) return 'var(--primary)';
    return card?.status === 'waiting' ? 'var(--border)' : 'var(--minimap-node)';
  };

  function drop(e: DragEvent) {
    const typeId = e.dataTransfer?.getData(PALETTE_MIME);
    if (!typeId || !ondropnode) return;
    e.preventDefault();
    const r = el.getBoundingClientRect();
    const x = (e.clientX - r.left - viewport.x) / viewport.zoom - NODE_WIDTH / 2;
    const y = (e.clientY - r.top - viewport.y) / viewport.zoom - nodeHeight(0) / 2;
    ondropnode(typeId, { x: Math.round(x), y: Math.round(y) });
  }
</script>

<div
  class="canvas"
  data-testid="node-canvas"
  role="application"
  bind:this={el}
  ondragover={(e) => editable && e.dataTransfer?.types.includes(PALETTE_MIME) && e.preventDefault()}
  ondrop={drop}
>
  <SvelteFlow
    bind:nodes
    bind:edges
    bind:viewport
    {nodeTypes}
    {edgeTypes}
    fitView={editable}
    fitViewOptions={{ padding: 0.08, minZoom: 0.6 }}
    minZoom={MIN_ZOOM}
    maxZoom={MAX_ZOOM}
    nodesConnectable={editable}
    deleteKey={editable ? ['Delete', 'Backspace'] : null}
    onnodedragstop={({ nodes: moved }) => onmove?.(Object.fromEntries(moved.map((n) => [n.id, n.position])))}
    onconnect={(c) => onconnect?.(c)}
    ondelete={({ nodes: gone, edges: cut }) => ondelete?.(gone.map((n) => n.id), cut.map((e) => e.id))}
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
    <Focus ids={focus} />
    <Panel position="bottom-left"><CanvasToolbar bind:minimap {onlist} {edit} /></Panel>
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
    font-size: 12px; line-height: 16px; font-weight: 700; letter-spacing: .06em; text-transform: uppercase; color: var(--text-3);
    pointer-events: none;
  }
  .sq { width: 6px; height: 6px; border-radius: 2px; }
</style>
