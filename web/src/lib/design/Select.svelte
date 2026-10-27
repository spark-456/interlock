<script lang="ts">
  import type { HTMLSelectAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';

  type Option = { value: string; label: string };
  type Props = HTMLSelectAttributes & { label?: string; options: Option[]; value?: string; error?: string };
  let { label, options, value = $bindable(''), error, id, ...rest }: Props = $props();
  const uid = $props.id();
  const selectId = $derived(id ?? `sel-${uid}`);
</script>

<div class="field">
  {#if label}<label for={selectId}>{label}</label>{/if}
  <div class="wrap">
    <select id={selectId} bind:value class:invalid={!!error} {...rest}>
      {#each options as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
    </select>
    <span class="chev"><Icon name="chevron-down" size={16} /></span>
  </div>
  {#if error}<span class="msg">{error}</span>{/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 6px; }
  label { font-size: 13px; font-weight: 600; color: var(--control-text); }
  .wrap { position: relative; }
  select {
    appearance: none; width: 100%; height: 40px; padding: 0 36px 0 12px; border-radius: var(--r-control);
    border: 1px solid var(--border-strong); background: var(--surface); font: 400 15px var(--font-sans); color: var(--text);
    box-shadow: var(--shadow-sm); cursor: pointer;
  }
  select:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; border-color: var(--primary); }
  select.invalid { border-color: var(--error); }
  select:disabled { background: var(--surface-3); color: var(--text-4); border-color: var(--border); cursor: not-allowed; }
  .chev { position: absolute; right: 12px; top: 12px; color: var(--text-3); pointer-events: none; display: flex; }
  .msg { font-size: 12px; color: var(--error-text); }
</style>
