<script lang="ts">
  // Labelled text input with error and hint text. `mono` for URLs, keys and ids.
  import type { HTMLInputAttributes } from 'svelte/elements';

  type Props = HTMLInputAttributes & { label?: string; error?: string; hint?: string; mono?: boolean; value?: string };
  let { label, error, hint, mono = false, value = $bindable(''), id, ...rest }: Props = $props();
  const uid = $props.id();
  const inputId = $derived(id ?? `in-${uid}`);
</script>

<div class="field">
  {#if label}<label for={inputId}>{label}</label>{/if}
  <input
    id={inputId}
    class:mono
    class:invalid={!!error}
    aria-invalid={error ? 'true' : undefined}
    aria-describedby={error || hint ? `${inputId}-msg` : undefined}
    bind:value
    {...rest}
  />
  {#if error}<span id="{inputId}-msg" class="msg error">{error}</span>
  {:else if hint}<span id="{inputId}-msg" class="msg">{hint}</span>{/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 6px; }
  label { font-size: 13px; font-weight: 600; color: var(--control-text); }
  input {
    height: 40px; padding: 0 12px; border-radius: var(--r-control); border: 1px solid var(--border-strong);
    font: 400 15px var(--font-sans); color: var(--text); background: var(--surface); box-shadow: var(--shadow-sm);
  }
  input.mono { font: 400 14px var(--font-mono); }
  input:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; border-color: var(--primary); }
  input.invalid { border-color: var(--error); }
  input:disabled { border-color: var(--border); background: var(--surface-3); color: var(--text-4); box-shadow: none; }
  .msg { font-size: 12px; line-height: 16px; color: var(--text-3); }
  .msg.error { color: var(--error-text); }
</style>
