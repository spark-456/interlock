<script lang="ts">
  // CopyField: plain, masked (only the tail shown) or reveal-once (amber, with warning).
  // "Copied" shows for 1.5 s.
  import Icon from './Icon.svelte';
  import { maskSecret } from './format';

  let { value, label, mode = 'plain' }: { value: string; label?: string; mode?: 'plain' | 'masked' | 'reveal-once' } = $props();
  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const shown = $derived(mode === 'masked' ? maskSecret(value) : value);

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      /* clipboard unavailable: still show feedback */
    }
    copied = true;
    clearTimeout(timer);
    timer = setTimeout(() => (copied = false), 1500);
  }
</script>

<div class="copy">
  {#if label}<span class="label">{label}</span>{/if}
  <div class="box {mode}">
    <span class="value">{shown}</span>
    <button type="button" class:copied onclick={copy} aria-label={copied ? 'Copied' : `Copy ${label ?? 'value'}`}>
      {#if copied}<Icon name="check" size={14} strokeWidth={3} />Copied{:else}Copy{/if}
    </button>
  </div>
  {#if mode === 'reveal-once'}<span class="warn">This key is shown once. Store it somewhere safe.</span>{/if}
</div>

<style>
  .copy { display: flex; flex-direction: column; gap: 8px; }
  .label { font-size: 12px; font-weight: 600; color: var(--text-3); }
  .box { display: flex; align-items: center; height: 44px; border-radius: var(--r-control); border: 1px solid var(--border-strong); overflow: hidden; background: var(--surface); }
  .value { flex: 1 1 auto; min-width: 0; padding: 0 12px; font: 400 13px var(--font-mono); color: var(--control-text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  button {
    display: inline-flex; align-items: center; gap: 6px; height: 44px; padding: 0 14px; border: 0;
    border-left: 1px solid var(--border-strong); background: var(--surface-3); color: var(--control-text);
    font: 600 13px var(--font-sans); cursor: pointer;
  }
  button.copied { background: var(--success-soft); color: var(--success-text); }
  button:focus-visible { outline: none; box-shadow: inset var(--focus-ring); }
  .reveal-once { border-color: var(--warning-line); background: var(--warning-soft); }
  .reveal-once .value { color: var(--text); }
  .reveal-once button { border-left-color: var(--warning-border); background: var(--surface); }
  .warn { font-size: 12px; line-height: 16px; color: var(--warning-text); }
</style>
