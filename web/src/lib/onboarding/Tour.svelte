<script lang="ts">
  // Guided tour (Q35): coach-marks over the real screens. A modal <dialog> keeps focus in the card and
  // ends the tour on Escape; a spotlight is cut around the step's element, or the card is centred.
  import { goto } from '$app/navigation';
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import type { FlowSummary } from '$lib/api';
  import { endTour, moveTour, onboarding } from './state.svelte';
  import { TOUR, place, stepPath, type Box, type TourStep } from './tour';

  const id = $props.id();
  let dialog: HTMLDialogElement | undefined = $state();
  let card: HTMLElement | undefined = $state();
  let target: HTMLElement | null = null;
  let spot = $state<Box | null>(null);
  let missing = $state(false);
  let cardW = $state(360);
  let cardH = $state(220);
  let view = $state({ w: 1280, h: 800 });

  const tour = $derived(onboarding.tour);
  const step = $derived(tour ? TOUR[tour.index] : undefined);
  const last = $derived(tour?.index === TOUR.length - 1);
  const at = $derived(place(spot, { w: cardW, h: cardH }, view));

  $effect(() => {
    if (!dialog) return;
    if (tour && !dialog.open) dialog.showModal();
    if (!tour && dialog.open) dialog.close();
  });

  $effect(() => {
    if (!step || !tour) return;
    let alive = true;
    void show(step, tour.flow, () => alive);
    return () => (alive = false);
  });

  $effect(() => {
    if (!tour) return;
    // Screens keep moving under the tour (data and fonts load, containers scroll, the window resizes),
    // so the spotlight follows its element every frame.
    let frame = requestAnimationFrame(function follow() {
      measure();
      frame = requestAnimationFrame(follow);
    });
    return () => cancelAnimationFrame(frame);
  });

  async function show(s: TourStep, flow: FlowSummary | null, alive: () => boolean) {
    const path = stepPath(s, flow);
    missing = path === null;
    target = null;
    // A new screen loads its data first, so its elements may take a moment to appear.
    const navigated = path !== null && path !== location.pathname;
    if (navigated) await goto(path);
    target = missing ? null : await find(s.target, navigated ? 4000 : 300, alive);
    if (!alive()) return;
    target?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    card?.querySelector<HTMLElement>('[data-next]')?.focus();
  }

  async function find(selectors: string[], ms: number, alive: () => boolean): Promise<HTMLElement | null> {
    const start = Date.now();
    while (alive()) {
      // The first selector gets a head start, so a fallback doesn't win while the screen's data loads.
      const late = Date.now() - start > Math.min(ms, 1500);
      for (const sel of late ? selectors : selectors.slice(0, 1)) {
        const el = document.querySelector<HTMLElement>(sel);
        if (el && el.offsetWidth > 0 && el.offsetHeight > 0) return el;
      }
      if (Date.now() - start > ms) break;
      await new Promise((r) => setTimeout(r, 100));
    }
    return null;
  }

  function measure() {
    if (view.w !== innerWidth || view.h !== innerHeight) view = { w: innerWidth, h: innerHeight };
    const r = target?.isConnected ? target.getBoundingClientRect() : undefined;
    const x = Math.max(0, (r?.x ?? 0) - 6);
    const y = Math.max(0, (r?.y ?? 0) - 6);
    const next = r ? { x, y, w: Math.min(r.right + 6, view.w) - x, h: Math.min(r.bottom + 6, view.h) - y } : null;
    if (next?.x !== spot?.x || next?.y !== spot?.y || next?.w !== spot?.w || next?.h !== spot?.h) spot = next;
  }

  function keys(e: KeyboardEvent) {
    if (e.key === 'ArrowRight') moveTour(1);
    else if (e.key === 'ArrowLeft') moveTour(-1);
  }
</script>

<dialog bind:this={dialog} class="tour" aria-labelledby="{id}-title" onclose={endTour} onkeydown={keys}>
  {#if tour && step}
    {#if spot}
      <div class="spot" style:left="{spot.x}px" style:top="{spot.y}px" style:width="{spot.w}px" style:height="{spot.h}px"></div>
    {:else}
      <div class="dim"></div>
    {/if}
    <div class="card" bind:this={card} bind:offsetWidth={cardW} bind:offsetHeight={cardH} style:left="{at.x}px" style:top="{at.y}px">
      <header>
        <span class="count">Tour · {tour.index + 1} of {TOUR.length}</span>
        <IconButton icon="x" label="End the tour" onclick={endTour} />
      </header>
      <div aria-live="polite">
        <h2 id="{id}-title">{step.title}</h2>
        <p>{step.body}</p>
        {#if missing}
          <p class="note"><Icon name="info" size={14} />{step.app ? 'Build an AI app' : 'Build a flow'} and this step points at the real screen.</p>
        {/if}
      </div>
      <footer>
        {#if !last}<Button variant="ghost" size="s" onclick={endTour}>Skip tour</Button>{/if}
        <span class="nav">
          {#if tour.index > 0}<Button size="s" onclick={() => moveTour(-1)}>Back</Button>{/if}
          <Button variant="primary" size="s" data-next onclick={() => moveTour(1)}>{last ? 'Done' : 'Next'}</Button>
        </span>
      </footer>
    </div>
  {/if}
</dialog>

<style>
  .tour {
    position: fixed; inset: 0; width: 100vw; height: 100vh; max-width: none; max-height: none; margin: 0; padding: 0;
    border: 0; background: transparent; overflow: hidden; color: var(--text);
  }
  .tour::backdrop { background: transparent; }
  .dim { position: absolute; inset: 0; background: var(--scrim); }
  .spot {
    position: absolute; border-radius: var(--r-card); box-shadow: 0 0 0 200vmax var(--scrim), var(--focus-ring);
    transition: left var(--dur-panel) var(--ease), top var(--dur-panel) var(--ease), width var(--dur-panel) var(--ease), height var(--dur-panel) var(--ease);
  }
  .card {
    position: absolute; width: min(360px, calc(100vw - 32px)); box-sizing: border-box; padding: var(--s-4) var(--s-5) var(--s-4);
    display: flex; flex-direction: column; gap: var(--s-3); background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--r-panel); box-shadow: var(--shadow-sheet);
    transition: left var(--dur-panel) var(--ease), top var(--dur-panel) var(--ease);
  }
  header { display: flex; align-items: center; justify-content: space-between; margin-bottom: calc(-1 * var(--s-2)); }
  .count { font: var(--t-small); font-weight: 600; color: var(--primary); }
  h2 { font: var(--t-h3); margin: 0 0 var(--s-2); }
  p { margin: 0; font-size: 14px; line-height: 21px; color: var(--text-2); }
  .note { display: flex; align-items: flex-start; gap: 6px; margin-top: var(--s-2); font: var(--t-small); color: var(--text-3); }
  .note :global(svg) { flex: none; margin-top: 2px; color: var(--primary); }
  footer { display: flex; align-items: center; gap: var(--s-2); }
  .nav { margin-left: auto; display: flex; gap: var(--s-2); }
  @media (prefers-reduced-motion: reduce) { .spot, .card { transition: none; } }
</style>
