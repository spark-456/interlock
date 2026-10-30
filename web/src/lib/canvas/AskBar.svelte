<script lang="ts">
  // Ask Interlock (ED-9 / AR-8): change requests in plain words, one after another. "@" or a click on
  // the canvas points at the steps to change; the page shows each returned diff on the canvas and this
  // bar holds the request, then the summary with Accept and Discard, then the changes kept so far.
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import { dropMention, matchSteps, mentionQuery, type Step } from './mentions';

  let {
    pending,
    summary,
    notes = [],
    applied = [],
    steps,
    mentions = $bindable([]),
    onask,
    onaccept,
    ondiscard,
    onclose
  }: {
    pending: boolean;
    /** Summary of the proposal on screen, or null before one arrives. */
    summary: string | null;
    /** What the proposal does, in the model's words. */
    notes?: string[];
    /** Summaries of the changes accepted while the bar was open. */
    applied?: string[];
    steps: Step[];
    /** Ids of the steps the request is about. */
    mentions?: string[];
    onask: (message: string) => void;
    onaccept: () => void;
    ondiscard: () => void;
    onclose: () => void;
  } = $props();

  let message = $state('');
  let active = $state(0);
  const query = $derived(mentionQuery(message));
  const options = $derived(query === null ? [] : matchSteps(steps, query, mentions));
  const name = (id: string) => steps.find((s) => s.id === id)?.name ?? id;

  function pick(step: Step) {
    mentions = [...mentions, step.id];
    message = dropMention(message);
    active = 0;
  }

  function keys(e: KeyboardEvent) {
    if (!options.length) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      active = (active + (e.key === 'ArrowDown' ? 1 : options.length - 1)) % options.length;
    } else if (e.key === 'Enter' || e.key === 'Tab') {
      e.preventDefault();
      pick(options[Math.min(active, options.length - 1)]!);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      message = dropMention(message);
    }
  }

  function submit(e: SubmitEvent) {
    e.preventDefault();
    if (!message.trim() || pending) return;
    onask(message.trim());
    message = '';
  }
</script>

<div class="ask" role="region" aria-label="Ask Interlock" data-testid="ask-bar">
  {#if applied.length}
    <ul class="applied" aria-label="Changes so far">
      {#each applied as a, i (i)}<li><Icon name="check" size={14} />{a}</li>{/each}
    </ul>
  {/if}
  {#if mentions.length}
    <div class="mentions" aria-label="Steps to change">
      {#each mentions as id (id)}
        <span class="mention">
          @{name(id)}
          <button type="button" aria-label="Don't change {name(id)}" onclick={() => (mentions = mentions.filter((m) => m !== id))}>
            <Icon name="x" size={12} />
          </button>
        </span>
      {/each}
    </div>
  {/if}
  <div class="row">
    <Icon name="sparkle" size={18} color="var(--primary)" />
    {#if summary}
      <div class="summary" aria-live="polite">
        <p><strong>Proposed:</strong> {summary}</p>
        {#each notes as note, i (i)}<p class="note">{note}</p>{/each}
      </div>
      <Button size="s" onclick={ondiscard}>Discard</Button>
      <Button size="s" variant="primary" onclick={onaccept}>Accept</Button>
    {:else}
      <form onsubmit={submit}>
        <input
          aria-label="Describe a change"
          role="combobox"
          aria-expanded={options.length > 0}
          aria-controls="ask-steps"
          aria-autocomplete="list"
          placeholder={applied.length ? 'Describe the next change…' : 'Describe a change; type @ to pick a step'}
          bind:value={message}
          onkeydown={keys}
          disabled={pending}
        />
        <Button size="s" variant="primary" type="submit" loading={pending} disabled={!message.trim()}>Propose</Button>
      </form>
      <IconButton icon="x" label="Close" onclick={onclose} />
    {/if}
  </div>
  {#if options.length && !summary}
    <ul class="options" id="ask-steps" role="listbox" aria-label="Steps">
      {#each options as s, i (s.id)}
        <li role="option" aria-selected={i === active}>
          <button type="button" tabindex="-1" onmousedown={(e) => e.preventDefault()} onclick={() => pick(s)}>{s.name}</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .ask {
    position: relative; display: flex; flex-direction: column; gap: 6px; width: min(720px, calc(100vw - 32px));
    padding: 8px 8px 8px 14px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card);
    box-shadow: var(--shadow-md);
  }
  .row { display: flex; align-items: center; gap: 10px; }
  form { flex: 1; display: flex; align-items: center; gap: 8px; min-width: 0; }
  input { flex: 1; min-width: 0; height: 32px; border: 0; outline: 0; background: transparent; font: inherit; font-size: 14px; color: var(--text); }
  .summary { flex: 1; min-width: 0; font-size: 14px; line-height: 20px; color: var(--text-2); }
  .summary p { margin: 0; }
  .summary strong { color: var(--text); font-weight: 600; }
  .note { font-size: 13px; color: var(--text-3); }
  .applied { display: flex; flex-direction: column; gap: 2px; margin: 0; padding: 0; list-style: none; font-size: 13px; color: var(--text-3); }
  .applied li { display: flex; align-items: center; gap: 6px; }
  .applied :global(svg) { color: var(--success); }
  .mentions { display: flex; flex-wrap: wrap; gap: 6px; }
  .mention {
    display: inline-flex; align-items: center; gap: 4px; height: 24px; padding: 0 4px 0 8px; border-radius: var(--r-chip);
    background: var(--primary-soft); color: var(--primary); font-size: 12px; font-weight: 600;
  }
  .mention button {
    display: inline-flex; align-items: center; justify-content: center; width: 18px; height: 18px; padding: 0; border: 0;
    border-radius: var(--r-chip); background: transparent; color: inherit; cursor: pointer;
  }
  .mention button:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .options {
    position: absolute; left: 36px; bottom: calc(100% + 4px); min-width: 220px; margin: 0; padding: 4px; list-style: none;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-md);
  }
  .options button {
    display: block; width: 100%; padding: 6px 10px; border: 0; border-radius: var(--r-chip); background: transparent;
    font: inherit; font-size: 14px; color: var(--text); text-align: left; cursor: pointer;
  }
  .options [aria-selected='true'] button, .options button:hover { background: var(--primary-soft); }
</style>
