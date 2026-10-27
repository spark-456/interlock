<script lang="ts">
  // Chip kinds: example (pill, pick a prompt), filter (selected / not), source (doc citation),
  // tool (mono, amber), diff-old / diff-new (before → after).
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  type Kind = 'example' | 'filter' | 'source' | 'tool' | 'diff-old' | 'diff-new';
  let {
    kind = 'example',
    selected = false,
    icon,
    onclick,
    children
  }: { kind?: Kind; selected?: boolean; icon?: string; onclick?: () => void; children: Snippet } = $props();
</script>

{#if onclick}
  <button type="button" class="chip {kind}" class:selected aria-pressed={kind === 'filter' ? selected : undefined} {onclick}>
    {#if icon}<Icon name={icon} size={14} />{/if}{@render children()}
  </button>
{:else}
  <span class="chip {kind}" class:selected>{#if icon}<Icon name={icon} size={14} />{/if}{@render children()}</span>
{/if}

<style>
  .chip { display: inline-flex; align-items: center; gap: 6px; white-space: nowrap; font-family: inherit; }
  button.chip { cursor: pointer; }
  .example { height: 30px; padding: 0 12px; border-radius: var(--r-pill); border: 1px solid var(--border-strong); background: var(--surface); font-size: 13px; font-weight: 500; box-shadow: var(--shadow-sm); color: var(--text); }
  .filter { height: 30px; padding: 0 12px; border-radius: var(--r-chip); border: 1px solid var(--border); background: var(--surface); color: var(--text-2); font-size: 13px; font-weight: 600; }
  .filter.selected { background: var(--primary-soft); color: var(--primary); border-color: var(--primary-line); }
  .source { height: 26px; padding: 0 10px; border-radius: var(--r-chip); background: var(--surface-2); color: var(--control-text); font-size: 12px; font-weight: 500; border: 0; }
  .tool { height: 26px; padding: 0 10px; border-radius: var(--r-chip); background: var(--warning-soft); color: var(--warning-text); font: 500 12px var(--font-mono); border: 0; }
  .diff-old { height: 22px; padding: 0 8px; border-radius: var(--r-chip); background: var(--error-soft); color: var(--error-text); font-size: 12px; font-weight: 600; text-decoration: line-through; }
  .diff-new { height: 22px; padding: 0 8px; border-radius: var(--r-chip); background: var(--success-soft); color: var(--success-text); font-size: 12px; font-weight: 600; }
  button.chip:focus-visible { outline: none; box-shadow: var(--focus-ring); }
</style>
