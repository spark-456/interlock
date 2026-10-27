<script lang="ts">
  // A text or long-text field that can hold `{{ … }}` references (ED-4). "Insert field" lists what
  // earlier steps produce and inserts it at the cursor; references show as readable chips below.
  import type { Flow } from '$lib/api/types';
  import Chip from '$lib/design/Chip.svelte';
  import Popover from '$lib/design/Popover.svelte';
  import type { Source } from './mapper';
  import { chipsOf } from './mapper';

  let {
    id,
    label,
    value,
    onchange,
    sources,
    flow,
    multiline = false,
    error,
    hint
  }: {
    id: string;
    label: string;
    value: string;
    onchange: (v: string) => void;
    sources: Source[];
    flow: Flow;
    multiline?: boolean;
    error?: string;
    hint?: string;
  } = $props();

  let el = $state<HTMLInputElement | HTMLTextAreaElement>();
  let open = $state(false);
  const chips = $derived(chipsOf(value, flow));
  const groups = $derived([...new Set(sources.map((s) => s.from))].map((from) => [from, sources.filter((s) => s.from === from)] as const));

  function insert(expr: string) {
    const at = el?.selectionStart ?? value.length;
    const end = el?.selectionEnd ?? at;
    onchange(value.slice(0, at) + expr + value.slice(end));
    open = false;
    el?.focus();
  }
</script>

<div class="field">
  <div class="head">
    <label for={id}>{label}</label>
    <Popover bind:open align="end">
      {#snippet trigger({ toggle })}
        <button type="button" class="insert" aria-expanded={open} onclick={toggle}>Insert field</button>
      {/snippet}
      <div class="picker" role="listbox" aria-label="Fields from earlier steps">
        {#each groups as [from, list] (from)}
          <div class="from">{from}</div>
          {#each list as s (s.expr)}
            <button type="button" role="option" aria-selected="false" class="opt" onclick={() => insert(s.expr)}>{s.field}</button>
          {/each}
        {:else}
          <p class="none">No earlier steps yet.</p>
        {/each}
      </div>
    </Popover>
  </div>
  {#if multiline}
    <textarea {id} rows="4" class:invalid={!!error} bind:this={el} {value} oninput={(e) => onchange(e.currentTarget.value)}></textarea>
  {:else}
    <input {id} class="mono" class:invalid={!!error} bind:this={el} {value} oninput={(e) => onchange(e.currentTarget.value)} />
  {/if}
  {#if chips.length}
    <div class="chips" aria-label="Mapped fields">{#each chips as c, i (i)}<Chip kind="source">{c}</Chip>{/each}</div>
  {/if}
  {#if error}<span class="msg error">{error}</span>{:else if hint}<span class="msg">{hint}</span>{/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 6px; }
  .head { display: flex; align-items: center; justify-content: space-between; }
  label { font-size: 13px; font-weight: 600; color: var(--text); }
  .insert { border: 0; background: none; padding: 0; font: inherit; font-size: 12px; font-weight: 600; color: var(--primary); cursor: pointer; }
  .insert:focus-visible { outline: none; box-shadow: var(--focus-ring); border-radius: 4px; }
  input, textarea {
    width: 100%; padding: 8px 10px; border: 1px solid var(--border-strong); border-radius: var(--r-control); background: var(--surface);
    color: var(--text); font: inherit; font-size: 14px; resize: vertical;
  }
  input.mono { font-family: var(--font-mono); font-size: 13px; height: 38px; }
  input:focus, textarea:focus { outline: none; box-shadow: var(--focus-ring); border-color: var(--primary); }
  .invalid { border-color: var(--error); }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .msg { font-size: 12px; color: var(--text-3); }
  .msg.error { color: var(--error-text); }
  .picker { width: 260px; max-height: 280px; overflow-y: auto; display: flex; flex-direction: column; }
  .from { padding: 8px 8px 4px; font-size: 11px; font-weight: 700; letter-spacing: .06em; text-transform: uppercase; color: var(--text-4); }
  .opt { text-align: left; border: 0; background: none; padding: 6px 8px; border-radius: 6px; font: 500 13px var(--font-mono); color: var(--text); cursor: pointer; }
  .opt:hover, .opt:focus-visible { background: var(--surface-2); outline: none; }
  .none { padding: 8px; font-size: 13px; color: var(--text-3); }
</style>
