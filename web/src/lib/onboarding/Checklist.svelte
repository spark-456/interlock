<script lang="ts">
  // Getting-started checklist (Q35). With `onhide` it is the card on Flows: dismissible, and gone once
  // every step is done. Without it, it sits inside Settings or the Help dialog under their heading.
  import { api } from '$lib/api';
  import Icon from '$lib/design/Icon.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import { checklist, type ChecklistStep } from './checklist';

  let { onhide }: { onhide?: () => void } = $props();
  const id = $props.id();
  let steps = $state.raw<ChecklistStep[] | null>(null);

  Promise.all([api.flows(), api.connections()]).then(([flows, connections]) => (steps = checklist(flows, connections)), () => {});

  const done = $derived(steps?.filter((s) => s.done).length ?? 0);
</script>

{#if steps && !(onhide && done === steps.length)}
  <section class="checklist" class:card={!!onhide} aria-labelledby={onhide ? `${id}-title` : undefined}>
    <header>
      {#if onhide}<h2 id="{id}-title">Getting started</h2>{/if}
      <span class="count" id="{id}-count">{done} of {steps.length} done</span>
      {#if onhide}<IconButton icon="x" label="Hide getting started" onclick={onhide} />{/if}
    </header>
    <div class="track" role="progressbar" aria-labelledby="{id}-count" aria-valuemin={0} aria-valuemax={steps.length} aria-valuenow={done}>
      <span style:width="{(done / steps.length) * 100}%"></span>
    </div>
    <ol>
      {#each steps as s (s.id)}
        <li class:done={s.done}>
          <a href={s.href}>
            <span class="tick" aria-hidden="true">{#if s.done}<Icon name="check" size={14} strokeWidth={2.6} />{/if}</span>
            <span class="text">
              <span class="label">{s.label}<span class="sr-only">{s.done ? ' (done)' : ' (to do)'}</span></span>
              <span class="hint">{s.hint}</span>
            </span>
          </a>
        </li>
      {/each}
    </ol>
  </section>
{/if}

<style>
  .checklist { display: flex; flex-direction: column; gap: var(--s-3); }
  .card {
    padding: var(--s-5); border-radius: var(--r-panel); background: var(--card-gradient); border: 1px solid var(--border);
    box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; align-items: center; gap: var(--s-3); }
  h2 { font: var(--t-h3); margin: 0; }
  .count { font: var(--t-small); color: var(--text-3); }
  header :global(.ib) { margin-left: auto; }
  .track { height: 6px; border-radius: var(--r-pill); background: var(--surface-2); overflow: hidden; }
  .track span { display: block; height: 100%; background: var(--primary); border-radius: inherit; transition: width var(--dur-panel) var(--ease); }
  ol { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--s-2); }
  .card ol { grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); }
  a {
    display: flex; align-items: flex-start; gap: var(--s-3); padding: var(--s-3); border-radius: var(--r-card);
    border: 1px solid var(--border); background: var(--surface); color: var(--text); text-decoration: none; height: 100%; box-sizing: border-box;
  }
  a:hover { background: var(--surface-3); border-color: var(--border-strong); }
  a:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .tick {
    flex: none; width: 22px; height: 22px; border-radius: 50%; border: 1.5px solid var(--border-strong);
    display: grid; place-items: center; color: var(--surface); box-sizing: border-box;
  }
  .done .tick { background: var(--primary); border-color: var(--primary); box-shadow: var(--shadow-check); }
  .text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .label { font-weight: 600; font-size: 14px; }
  .done .label { color: var(--text-3); text-decoration: line-through; }
  .hint { font: var(--t-small); color: var(--text-3); }
  @media (prefers-reduced-motion: reduce) { .track span { transition: none; } }
</style>
