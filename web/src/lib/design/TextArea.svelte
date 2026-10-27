<script lang="ts">
  import type { HTMLTextareaAttributes } from 'svelte/elements';

  type Props = HTMLTextareaAttributes & { label?: string; error?: string; hint?: string; value?: string };
  let { label, error, hint, value = $bindable(''), id, rows = 4, ...rest }: Props = $props();
  const uid = $props.id();
  const areaId = $derived(id ?? `ta-${uid}`);
</script>

<div class="field">
  {#if label}<label for={areaId}>{label}</label>{/if}
  <textarea id={areaId} {rows} class:invalid={!!error} aria-invalid={error ? 'true' : undefined} bind:value {...rest}></textarea>
  {#if error}<span class="msg error">{error}</span>{:else if hint}<span class="msg">{hint}</span>{/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 6px; }
  label { font-size: 13px; font-weight: 600; color: var(--control-text); }
  textarea {
    padding: 10px 12px; border-radius: var(--r-control); border: 1px solid var(--border-strong); resize: vertical;
    font: var(--t-body); color: var(--text); background: var(--surface); box-shadow: var(--shadow-sm);
  }
  textarea:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; border-color: var(--primary); }
  textarea.invalid { border-color: var(--error); }
  textarea:disabled { border-color: var(--border); background: var(--surface-3); color: var(--text-4); }
  .msg { font-size: 12px; line-height: 16px; color: var(--text-3); }
  .msg.error { color: var(--error-text); }
</style>
