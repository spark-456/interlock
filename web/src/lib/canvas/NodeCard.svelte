<script lang="ts">
  // NodeCard (DESIGN.md anatomy): 264 px; header = 32 px tile + title + type; body = stat rows;
  // footer = StatusPill. Pop shadow; hover lifts 2 px, selected 3 px + ring, dragging 6 px + tilt.
  import { Handle, Position, type Node, type NodeProps } from '@xyflow/svelte';
  import AppTile from '$lib/design/AppTile.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import StatusPill, { type PillTone } from '$lib/design/StatusPill.svelte';
  import { toneVars } from './categories';
  import { NODE_WIDTH, nodeHeight } from './geometry';
  import { pillLabel, type CardView } from './types';

  let { data, selected = false, dragging = false, isConnectable = false }: NodeProps<Node<{ card: CardView }, 'card'>> =
    $props();
  const card = $derived(data.card);
  const vars = $derived(toneVars(card.tone));
  const tone = $derived<PillTone>(card.status);
  const mono = $derived(card.tone === 'tool');
</script>

<div
  class="card {card.status}"
  class:selected
  class:dragging
  style:width="{NODE_WIDTH}px"
  style:height="{nodeHeight(card.stats.length)}px"
  aria-label="{card.subtitle}: {card.title}, {pillLabel(card)}"
  data-testid="node-{card.id}"
  data-diff={card.diff}
>
  {#if card.status === 'building'}
    <div class="bar" aria-hidden="true"><div style:width="{Math.round((card.progress ?? 0.7) * 100)}%"></div></div>
  {/if}
  <div class="head">
    {#if card.tone === 'app' && card.app}
      <AppTile name={card.app} size={32} />
    {:else}
      <span class="tile" style:--tile={vars.tile} style:border-color={vars.edge}>
        <Icon name={card.icon} size={18} color={vars.icon} />
      </span>
    {/if}
    <div class="titles">
      <div class="title">{card.title}</div>
      <div class="type">{card.subtitle}</div>
    </div>
  </div>
  {#if card.stats.length}
    <div class="stats">
      {#each card.stats as s (s.label)}
        <div class="row"><span class="k" class:mono>{s.label}</span><span class="v">{s.value}</span></div>
      {/each}
    </div>
  {/if}
  <div class="foot"><StatusPill {tone} label={pillLabel(card)} /></div>
  <Handle type="target" position={Position.Left} class="il-handle {card.showIn ? '' : 'hidden'}" isConnectable={isConnectable && card.showIn} />
  <Handle type="source" position={Position.Right} class="il-handle {card.showOut ? '' : 'hidden'}" isConnectable={isConnectable && card.showOut} />
  <Handle type="source" id="bottom" position={Position.Bottom} class="il-handle hidden" isConnectable={false} />
  <Handle type="target" id="top" position={Position.Top} class="il-handle hidden" isConnectable={false} />
</div>

<style>
  .card {
    position: relative; padding: 14px; border-radius: var(--r-card); background: var(--card-gradient);
    border: 1px solid var(--border); box-shadow: var(--shadow-pop); display: flex; flex-direction: column; gap: 12px;
    color: var(--text); cursor: grab; transition: box-shadow var(--dur-pill) ease, transform var(--dur-pill) ease, opacity var(--dur-pill) ease;
    animation: enter var(--dur-node-enter) ease-out;
  }
  .card:hover { box-shadow: var(--shadow-pop-hover); transform: translateY(-2px); }
  .card.selected { box-shadow: var(--shadow-pop-selected); transform: translateY(-3px); }
  .card.dragging { box-shadow: var(--shadow-pop-drag); transform: translateY(-6px) rotate(-0.6deg); cursor: grabbing; }
  .card.waiting { opacity: .6; border-style: dashed; border-color: var(--border-strong); box-shadow: none; }
  .card.failed { border-color: var(--error); box-shadow: 0 0 0 1px var(--error), var(--shadow-pop); }
  .card[data-diff='added'] { border-color: var(--success); box-shadow: 0 0 0 2px var(--success), var(--shadow-pop); }
  .card[data-diff='changed'] { border-color: var(--warning); box-shadow: 0 0 0 2px var(--warning), var(--shadow-pop); }
  .card[data-diff='removed'] { opacity: .55; border-style: dashed; }
  @keyframes enter { from { opacity: 0; transform: translateY(8px); } }

  .bar { position: absolute; left: 12px; right: 12px; top: -1px; height: 3px; border-radius: 0 0 3px 3px; background: var(--primary-track); overflow: hidden; }
  .bar div { height: 3px; background: var(--primary); transition: width var(--dur-panel) ease; }

  .head { display: flex; align-items: center; gap: 10px; }
  .tile {
    flex: 0 0 32px; width: 32px; height: 32px; border-radius: 9px; border: 1px solid;
    background: linear-gradient(180deg, var(--surface) -40%, var(--tile) 70%); box-shadow: var(--shadow-tile);
    display: flex; align-items: center; justify-content: center;
  }
  .titles { min-width: 0; flex: 1 1 auto; }
  .title { font-size: 14px; line-height: 18px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .type { font-size: 12px; line-height: 16px; font-weight: 500; color: var(--text-3); }

  .stats { display: flex; flex-direction: column; gap: 6px; padding-top: 10px; border-top: 1px solid var(--surface-2); }
  .row { display: flex; justify-content: space-between; align-items: center; gap: 8px; height: 16px; font-size: 12px; line-height: 16px; }
  .k { color: var(--text-2); font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .k.mono { font-family: var(--font-mono); }
  .v { font-family: var(--font-mono); font-weight: 500; color: var(--text); white-space: nowrap; }
  .foot { display: flex; margin-top: auto; }

  .card :global(.il-handle) {
    width: 10px; height: 10px; min-width: 0; min-height: 0; border-radius: 50%;
    background: var(--surface); border: 1.5px solid var(--border-strong);
  }
  .card:hover :global(.il-handle), .card.selected :global(.il-handle) { border-color: var(--primary); }
  .card :global(.il-handle.hidden) { opacity: 0; pointer-events: none; }
</style>
