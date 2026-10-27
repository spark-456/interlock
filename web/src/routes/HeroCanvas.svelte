<script lang="ts">
  // The landing hero: the recorded Larkspur build replayed on the read-only canvas, looping. The
  // recording ships with the page, so it plays without a backend or a sign-in.
  import { onDestroy } from 'svelte';
  import type { BuildEvent } from '$lib/api/types';
  import NodeCanvas from '$lib/canvas/NodeCanvas.svelte';
  import { initialBuildState, progress, reduce } from '$lib/canvas/buildState';
  import { cardViews, edgeViews, labelViews } from '$lib/canvas/views';
  import { clock } from '$lib/design/format';
  import StatusPill from '$lib/design/StatusPill.svelte';

  let build = $state.raw(initialBuildState());
  let timers: ReturnType<typeof setTimeout>[] = [];
  const reduced = typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;

  function play(events: BuildEvent[]) {
    build = initialBuildState();
    if (reduced) {
      build = events.reduce(reduce, build);
      return;
    }
    const t0 = Date.parse(events[0]?.ts ?? '');
    for (const e of events) timers.push(setTimeout(() => (build = reduce(build, e)), (Date.parse(e.ts) - t0) / 2));
    const end = (Date.parse(events.at(-1)?.ts ?? '') - t0) / 2;
    timers.push(setTimeout(() => play(events), end + 6000));
  }

  import('$fixtures/streams/builds/bld_larkspur.json').then((m) => play(m.default as BuildEvent[]));
  onDestroy(() => timers.forEach(clearTimeout));

  const cards = $derived(cardViews(build));
  const edges = $derived(edgeViews(build));
  const labels = $derived(labelViews(build, cards));
  const log = $derived(build.log.slice(-4));
</script>

<figure class="frame" aria-label="Interlock building the Larkspur Support assistant">
  <header>
    <span class="dots" aria-hidden="true"><i></i><i></i><i></i></span>
    <strong>Building {build.flow.name ?? 'Larkspur Support'}</strong>
    <StatusPill tone={build.status === 'ready' ? 'ready' : 'building'} label={build.status === 'ready' ? 'Ready' : `Building · ${clock(build.elapsed)}`} />
    <span class="url">interlock.example/build/larkspur</span>
  </header>
  <div class="canvas"><NodeCanvas {cards} {edges} {labels} /></div>
  <figcaption class="log" aria-hidden="true">
    <span class="head"><strong>Build log</strong><span class="mono">{clock(build.elapsed)}</span></span>
    <span class="bar"><span style:width="{Math.round(progress(build) * 100)}%"></span></span>
    {#each log as l (l.t + l.text)}<span class="line"><span class="mono">{clock(l.t)}</span> {l.text}</span>{/each}
  </figcaption>
</figure>

<style>
  .frame {
    position: relative; margin: 0; overflow: hidden; border: 1px solid var(--border); border-radius: var(--r-panel);
    background: var(--surface); box-shadow: var(--shadow-lg);
  }
  header { display: flex; align-items: center; gap: var(--s-3); padding: var(--s-2) var(--s-4); border-bottom: 1px solid var(--border); font: var(--t-small); }
  .dots { display: flex; gap: var(--s-1); }
  .dots i { width: 8px; height: 8px; border-radius: 50%; background: var(--border-strong); }
  .url { margin-left: auto; color: var(--text-4); font-family: var(--font-mono); }
  .canvas { position: relative; height: 460px; pointer-events: none; }
  .log {
    position: absolute; right: var(--s-5); bottom: var(--s-5); width: 300px; padding: var(--s-3) var(--s-4); display: flex; flex-direction: column; gap: var(--s-2);
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-pop); font: var(--t-small);
  }
  .head { display: flex; justify-content: space-between; }
  .bar { height: 4px; border-radius: 2px; background: var(--primary-track); overflow: hidden; }
  .bar span { display: block; height: 100%; background: var(--primary); transition: width var(--dur-panel) ease; }
  .line { color: var(--text-2); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .mono { font-family: var(--font-mono); color: var(--text-3); }
  @media (max-width: 767px) {
    .canvas { height: 320px; }
    .url { display: none; }
    .log { left: var(--s-3); right: var(--s-3); bottom: var(--s-3); width: auto; }
  }
</style>
