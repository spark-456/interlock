<script lang="ts">
  // Chat playground (design S5): each message is a test run of the app; its run events stream in as
  // activity, then the answer with sources. A source chip opens the passage beside the chat.
  import { onDestroy, tick } from 'svelte';
  import { page } from '$app/state';
  import { api, runEvents, type Flow } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import TopBar from '$lib/design/TopBar.svelte';
  import TurnView from './TurnView.svelte';
  import { newTurn, passageFor, reduceTurn, suggestions, type Turn } from './chat';

  const flowId = page.params.id ?? '';
  const speed = Number(page.url.searchParams.get('speed') ?? import.meta.env.VITE_FIXTURE_SPEED ?? 1) || 1;
  let flow = $state.raw<Flow | null>(null);
  let turns = $state.raw<Turn[]>([]);
  let draft = $state('');
  let source = $state<{ turn: number; sourceId: string } | null>(null);
  let log: HTMLOListElement | undefined = $state();
  let sub: { close(): void } | undefined;

  api.flow(flowId).then((f) => (flow = f), () => {});
  onDestroy(() => sub?.close());

  const busy = $derived(turns.at(-1)?.status === 'running');
  const shown = $derived(source ? passageFor(turns[source.turn]!, source.sourceId) : undefined);

  async function ask(text: string) {
    const question = text.trim();
    if (!question || busy || !flow) return;
    draft = '';
    const history = turns.flatMap((t) => [
      { role: 'user', content: t.question },
      ...(t.answer ? [{ role: 'assistant', content: t.answer }] : [])
    ]);
    const index = turns.length;
    turns = [...turns, newTurn(question)];
    const update = (f: (t: Turn) => Turn) => (turns = turns.map((t, i) => (i === index ? f(t) : t)));
    try {
      const messages = [...history, { role: 'user', content: question }];
      const { runId } = await api.testRun(flowId, [{ message: question, messages }]);
      sub?.close();
      sub = runEvents(runId, (e) => flow && update((t) => reduceTurn(t, e, flow!)), { speed });
    } catch (e) {
      update((t) => ({ ...t, status: 'failed', error: e instanceof Error ? e.message : 'The app could not answer.' }));
    }
    await tick();
    log?.lastElementChild?.scrollIntoView({ behavior: 'smooth', block: 'end' });
  }

  function reset() {
    sub?.close();
    turns = [];
    source = null;
  }
</script>

<svelte:head><title>Playground · {flow?.name ?? 'Interlock'}</title></svelte:head>

<div class="screen">
  <TopBar title={flow?.name ?? 'Playground'} crumb={{ href: `/flows/${flowId}`, label: 'Flow' }} back="/flows/{flowId}">
    {#snippet status()}<StatusPill tone="building" label="Playground" size={26} />{/snippet}
    {#snippet actions()}
      <Button size="s" icon="repeat" disabled={!turns.length} onclick={reset}>New chat</Button>
      <Button size="s" variant="primary" href="/flows/{flowId}/publish">Connect</Button>
    {/snippet}
  </TopBar>
  <div class="body">
    <section class="chat" aria-label="Chat">
      <ol class="log" bind:this={log}>
        {#each turns as turn, i (i)}
          <TurnView {turn} onsource={(sourceId) => (source = { turn: i, sourceId })} onask={ask} />
        {:else}
          <li class="empty">
            <Icon name="chat" size={28} />
            <p>Ask {flow?.name ?? 'the app'} anything. Each message is a test run: you see how it routes, which tools it calls and what it reads.</p>
          </li>
        {/each}
      </ol>
      <form onsubmit={(e) => (e.preventDefault(), ask(draft))}>
        <input bind:value={draft} placeholder="Ask {flow?.name ?? 'the app'}…" aria-label="Message" autocomplete="off" />
        <Button variant="primary" icon="send" type="submit" disabled={busy || !draft.trim()}>Send</Button>
      </form>
      <p class="note">Playground messages are test runs: nothing is sent or posted for real, and they count toward your caps.</p>
    </section>

    {#if shown && source}
      <aside class="source" aria-label="Source">
        <header><h2>Source</h2><IconButton icon="x" label="Close source" onclick={() => (source = null)} /></header>
        <p class="title"><Icon name="docs" size={16} /> {shown.passage.title}{#if shown.passage.section}<span>{shown.passage.section}</span>{/if}</p>
        <blockquote>{shown.passage.text}</blockquote>
        <dl>
          <div><dt>Rank</dt><dd>{shown.rank} of {shown.of}</dd></div>
          <div><dt>Match</dt><dd>{shown.passage.score.toFixed(2)}</dd></div>
        </dl>
        {#if shown.passage.url}<a href={shown.passage.url} target="_blank" rel="noopener noreferrer">Open the page</a>{/if}
      </aside>
    {/if}

    <aside class="try" aria-label="Try asking">
      <h2>Try asking</h2>
      <ul>
        {#each flow ? suggestions(flow) : [] as q (q)}
          <li><button type="button" disabled={busy} onclick={() => ask(q)}><Icon name="chat" size={14} /> <span>{q}</span> <Icon name="chevron-right" size={14} /></button></li>
        {/each}
      </ul>
    </aside>
  </div>
</div>

<style>
  .screen { display: flex; flex-direction: column; height: 100vh; min-height: 0; }
  .body { flex: 1 1 auto; min-height: 0; display: flex; }
  .chat { flex: 1; min-width: 0; display: flex; flex-direction: column; padding: var(--s-5) var(--s-6) var(--s-4); gap: var(--s-3); }
  .log { flex: 1; overflow-y: auto; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--s-6); }
  .empty { list-style: none; margin: auto; max-width: 420px; text-align: center; color: var(--text-3); display: flex; flex-direction: column; align-items: center; gap: var(--s-2); }
  .empty p { margin: 0; }
  form {
    display: flex; gap: var(--s-2); padding: var(--s-2); align-items: center;
    background: var(--surface); border: 1px solid var(--border-strong); border-radius: var(--r-panel); box-shadow: var(--shadow-md);
  }
  input { flex: 1; border: 0; outline: none; padding: var(--s-2) var(--s-3); font: inherit; color: var(--text); background: transparent; }
  form:focus-within { outline: 2px solid var(--primary); outline-offset: 1px; }
  .note { margin: 0; color: var(--text-3); font: var(--t-small); text-align: center; }
  aside { width: 320px; flex: none; overflow-y: auto; padding: var(--s-5); border-left: 1px solid var(--border); background: var(--surface); display: flex; flex-direction: column; gap: var(--s-4); }
  aside h2 { font: var(--t-h3); margin: 0; }
  .source header { display: flex; justify-content: space-between; align-items: center; }
  .title { margin: 0; display: flex; flex-wrap: wrap; align-items: center; gap: var(--s-2); font-weight: 600; }
  .title span { width: 100%; font: var(--t-small); font-weight: 400; color: var(--text-3); }
  blockquote { margin: 0; padding: var(--s-3) var(--s-4); border-radius: var(--r-card); background: var(--primary-soft); color: var(--text); }
  dl { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-3); margin: 0; }
  dt { color: var(--text-3); font: var(--t-small); }
  dd { margin: var(--s-1) 0 0; font-family: var(--font-mono); }
  .source a { color: var(--primary); font-weight: 600; }
  .try ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--s-1); }
  .try button {
    width: 100%; display: flex; align-items: center; gap: var(--s-2); padding: var(--s-3); text-align: left; font: inherit; color: var(--text);
    border: 0; border-radius: var(--r-card); background: none; cursor: pointer;
  }
  .try button:hover:not(:disabled) { background: var(--surface-3); }
  .try button span { flex: 1; }
  .try button :global(svg:first-child) { color: var(--primary); }
  @media (max-width: 1023px) {
    .try { display: none; }
  }
  @media (max-width: 767px) {
    .chat { padding: var(--s-3); }
    .source { position: fixed; inset: auto 0 0 0; width: auto; max-height: 60vh; border-left: 0; border-top: 1px solid var(--border); box-shadow: var(--shadow-lg); z-index: 10; }
  }
</style>
