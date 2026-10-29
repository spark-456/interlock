<script lang="ts">
  // List view (M2): cards grouped by column, connected by short vertical lines. Used on mobile
  // (<768 px) and as the canvas's accessible alternative.
  import Icon from '$lib/design/Icon.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import { toneVars } from './categories';
  import { pillLabel, type GroupView } from './types';
  import { groupStatus } from './views';

  let { groups, onselect }: { groups: GroupView[]; onselect?: (id: string) => void } = $props();
</script>

<div class="list" data-testid="node-list">
  {#each groups as g, gi (g.id)}
    {#if gi > 0}<div class="link" aria-hidden="true"><span class:queued={g.cards.every((c) => c.status === 'waiting')}></span></div>{/if}
    <section aria-label={g.label}>
      <h3><span class="sq" style:background={toneVars(g.tone).icon}></span>{g.label}<span class="gs">{groupStatus(g)}</span></h3>
      <div class="cards">
        {#each g.cards as c (c.id)}
          {@const v = toneVars(c.tone)}
          <button type="button" class="item {c.status}" aria-label="{c.title}, {pillLabel(c)}. Open details" onclick={() => onselect?.(c.id)}>
            {#if c.status === 'building'}<span class="bar"><span style:width="{Math.round((c.progress ?? 0.7) * 100)}%"></span></span>{/if}
            <span class="tile" style:--tile={v.tile} style:border-color={v.edge}><Icon name={c.icon} size={20} color={v.icon} /></span>
            <span class="text">
              <span class="t">{c.title}</span>
              <span class="s">{c.summary ?? c.subtitle}</span>
            </span>
            <StatusPill tone={c.status} label={pillLabel(c)} />
          </button>
        {/each}
      </div>
    </section>
  {/each}
</div>

<style>
  .list { display: flex; flex-direction: column; }
  .link { height: 22px; display: flex; justify-content: center; }
  .link span { width: 2px; height: 22px; border-radius: 1px; background: var(--edge); }
  .link span.queued { background: var(--edge-queued); }
  h3 {
    display: flex; align-items: center; gap: 8px; padding: 6px 2px 8px; font-size: 12px; line-height: 16px;
    font-weight: 700; letter-spacing: .06em; text-transform: uppercase; color: var(--text-3);
  }
  .sq { width: 6px; height: 6px; border-radius: 2px; }
  .gs { margin-left: auto; font-weight: 600; letter-spacing: 0; text-transform: none; color: var(--text-3); }
  .cards { display: flex; flex-direction: column; gap: 8px; }
  .item {
    position: relative; width: 100%; min-height: 64px; display: flex; align-items: center; gap: 12px; padding: 12px;
    text-align: left; border-radius: var(--r-card); background: var(--card-gradient); border: 1px solid var(--border);
    box-shadow: var(--shadow-pop-sm); color: inherit; cursor: pointer; animation: enter var(--dur-node-enter) ease-out;
  }
  .item.waiting { background: var(--surface-2); border-style: dashed; border-color: var(--border-strong); box-shadow: none; }
  .item.building { border: 1.5px solid var(--primary); box-shadow: var(--shadow-pop-active); }
  .item.failed { border-color: var(--error); }
  .item:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .bar { position: absolute; left: 12px; right: 12px; top: -1px; height: 3px; border-radius: 0 0 3px 3px; background: var(--primary-track); overflow: hidden; }
  .bar span { display: block; height: 3px; background: var(--primary); }
  .tile {
    flex: 0 0 36px; width: 36px; height: 36px; border-radius: var(--r-tile); border: 1px solid;
    background: linear-gradient(180deg, var(--surface) -40%, var(--tile) 70%); box-shadow: var(--shadow-tile);
    display: flex; align-items: center; justify-content: center;
  }
  .text { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; }
  .t { font-size: 14px; line-height: 19px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .s { font: 400 12px/17px var(--font-mono); color: var(--text-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  @keyframes enter { from { opacity: 0; transform: translateY(8px); } }
</style>
