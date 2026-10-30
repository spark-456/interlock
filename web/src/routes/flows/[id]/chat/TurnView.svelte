<script lang="ts">
  // One question and its answer: activity chips (route, tools, searches, the path), the answer,
  // source chips that open the passage, suggested actions, and the guardrail note when one fired.
  import Button from '$lib/design/Button.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import Spinner from '$lib/design/Spinner.svelte';
  import { clock } from '$lib/design/format';
  import { markdown } from '$lib/design/markdown';
  import type { Turn } from './chat';

  let { turn, onsource, onask }: { turn: Turn; onsource: (sourceId: string) => void; onask: (message: string) => void } = $props();
  let showPath = $state(false);
  const searched = $derived(new Set(turn.passages.map((p) => p.sourceId)).size);
</script>

<li class="turn">
  <p class="user">{turn.question}</p>
  <div class="activity" aria-label="Activity">
    <button type="button" class="toggle" aria-expanded={showPath} onclick={() => (showPath = !showPath)}>
      <Icon name={showPath ? 'chevron-down' : 'chevron-right'} size={14} /> Activity
    </button>
    {#if turn.route}<Chip kind="source" icon="split">Routed to {turn.route}</Chip>{/if}
    {#each turn.tools as t (t)}<Chip kind="tool">{t}</Chip>{/each}
    {#if turn.passages.length}<Chip kind="source" icon="book">Searched {searched} sources · {turn.passages.length} passages</Chip>{/if}
  </div>
  {#if showPath}
    <ol class="path" aria-label="Path through the flow">
      {#each turn.path as p (p.nodeId)}<li>{p.name}{#if p.ms !== undefined}<span>{clock(p.ms)}</span>{/if}</li>{/each}
    </ol>
  {/if}
  <article class="answer" class:blocked={!!turn.guardrail} aria-live="polite">
    {#if turn.answer}
      {#if turn.by}<p class="by">{turn.by.name}{#if turn.by.ms !== undefined} · <span class="mono">{(turn.by.ms / 1000).toFixed(1)} s</span>{/if}</p>{/if}
      <!-- markdown() escapes the answer before adding its own tags. -->
      <div class="text">{@html markdown(turn.answer)}</div>
      {#if turn.sources.length}
        <div class="chips">
          {#each turn.sources as s (s.sourceId)}
            <Chip kind="source" icon="docs" onclick={() => onsource(s.sourceId)}>{s.title}{s.section ? ` · ${s.section}` : ''}</Chip>
          {/each}
        </div>
      {/if}
      {#if turn.guardrail}<p class="guard"><Icon name="shield" size={14} /> Guardrail: {turn.guardrail}</p>{/if}
      {#if turn.actions.length && turn.status === 'done'}
        <div class="actions">
          {#each turn.actions as a, i (a.label)}
            <Button size="s" variant={i === 0 ? 'primary' : 'secondary'} onclick={() => onask(a.message)}>{a.label}</Button>
          {/each}
        </div>
        <p class="hint">Suggested by the assistant</p>
      {/if}
    {:else if turn.error}
      <p class="error" role="alert">{turn.error}</p>
    {:else}
      <p class="thinking"><Spinner size={14} /> Thinking…</p>
    {/if}
  </article>
</li>

<style>
  .turn { display: flex; flex-direction: column; gap: var(--s-2); list-style: none; }
  .user {
    align-self: flex-end; max-width: 80%; margin: 0; padding: var(--s-3) var(--s-4); white-space: pre-wrap;
    border-radius: var(--r-panel); background: var(--primary); color: var(--surface);
  }
  .activity { display: flex; flex-wrap: wrap; align-items: center; gap: var(--s-2); }
  .toggle { display: inline-flex; align-items: center; gap: var(--s-1); border: 0; background: none; padding: 0; font: var(--t-small); color: var(--text-2); cursor: pointer; }
  .path { margin: 0; padding: var(--s-2) var(--s-4); display: flex; flex-wrap: wrap; gap: var(--s-1) var(--s-3); font: var(--t-small); color: var(--text-2); }
  .path li span { margin-left: var(--s-1); color: var(--text-3); font-family: var(--font-mono); }
  .answer {
    max-width: 88%; padding: var(--s-4) var(--s-5); display: flex; flex-direction: column; gap: var(--s-3);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  .answer.blocked { border-style: dashed; }
  .by { margin: 0; font: var(--t-small); font-weight: 600; color: var(--primary); }
  .mono { font-family: var(--font-mono); color: var(--text-3); font-weight: 400; }
  .text { overflow-wrap: anywhere; }
  .text :global(:is(p, ul, ol, pre, h4)) { margin: 0 0 var(--s-2); }
  .text :global(:last-child) { margin-bottom: 0; }
  .text :global(:is(ul, ol)) { padding-left: var(--s-5); }
  .text :global(h4) { font-weight: 600; }
  .text :global(code) { font-family: var(--font-mono); font-size: 0.9em; background: var(--surface-2); border-radius: var(--r-chip); padding: 0 4px; }
  .text :global(pre) { overflow-x: auto; padding: var(--s-2) var(--s-3); background: var(--surface-2); border-radius: var(--r-card); }
  .text :global(pre code) { padding: 0; background: none; }
  .text :global(a) { color: var(--primary); }
  .text :global(table) { border-collapse: collapse; margin: 0 0 var(--s-2); font: var(--t-small); }
  .text :global(:is(th, td)) { border: 1px solid var(--border); padding: var(--s-1) var(--s-2); text-align: left; vertical-align: top; }
  .text :global(th) { background: var(--surface-2); font-weight: 600; }
  .chips, .actions { display: flex; flex-wrap: wrap; gap: var(--s-2); }
  .guard { margin: 0; display: inline-flex; align-items: center; gap: var(--s-1); color: var(--error-text); font: var(--t-small); }
  .hint { margin: 0; color: var(--text-3); font: var(--t-small); }
  .thinking { margin: 0; display: flex; align-items: center; gap: var(--s-2); color: var(--text-3); }
  .error { margin: 0; color: var(--error-text); }
</style>
