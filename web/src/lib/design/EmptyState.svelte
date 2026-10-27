<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let { icon = 'inbox', title, text, actions }: { icon?: string; title: string; text?: string; actions?: Snippet } = $props();
</script>

<section class="empty">
  <span class="mark"><Icon name={icon} size={24} /></span>
  <h2>{title}</h2>
  {#if text}<p>{text}</p>{/if}
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</section>

<style>
  .empty {
    display: flex; flex-direction: column; align-items: center; gap: var(--s-3); padding: var(--s-8) var(--s-6); text-align: center;
    border: 1px dashed var(--border-strong); border-radius: var(--r-panel);
  }
  .mark { display: grid; place-items: center; width: 48px; height: 48px; border-radius: var(--r-panel); background: var(--primary-soft); color: var(--primary); }
  h2 { font: var(--t-h3); margin: 0; }
  p { margin: 0; max-width: 420px; color: var(--text-2); }
  .actions { display: flex; gap: var(--s-2); flex-wrap: wrap; justify-content: center; }
</style>
