<script lang="ts">
  // Page banners: offline, cap reached.
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let { kind, children, action }: { kind: 'offline' | 'cap'; children: Snippet; action?: Snippet } = $props();
  const icon = $derived(kind === 'offline' ? 'alert' : 'bolt');
</script>

<div class="banner {kind}" role="alert">
  <Icon name={icon} size={16} />
  <span class="text">{@render children()}</span>
  {#if action}{@render action()}{/if}
</div>

<style>
  .banner { display: flex; align-items: center; gap: var(--s-2); min-height: 40px; padding: 8px 16px; font-size: 13px; font-weight: 500; }
  .text { flex: 1; }
  .offline { background: var(--surface-2); color: var(--control-text); }
  .cap { background: var(--warning-soft); color: var(--warning-text); }
</style>
