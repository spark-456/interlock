<script lang="ts">
  // Renders the toast queue from `toasts.svelte.ts`. Mounted once in the root layout.
  import Icon from './Icon.svelte';
  import { toasts, dismiss } from './toasts.svelte';
</script>

<div class="stack" aria-live="polite">
  {#each toasts.list as t (t.id)}
    <div class="toast {t.tone}" role="status">
      <Icon name={t.tone === 'error' ? 'alert' : t.tone === 'success' ? 'check' : 'info'} size={16} />
      <span>{t.message}</span>
      <button type="button" aria-label="Dismiss" onclick={() => dismiss(t.id)}><Icon name="x" size={14} /></button>
    </div>
  {/each}
</div>

<style>
  .stack { position: fixed; right: var(--s-4); bottom: var(--s-4); z-index: 50; display: flex; flex-direction: column; gap: var(--s-2); }
  .toast {
    display: flex; align-items: center; gap: var(--s-2); min-width: 260px; max-width: 380px; padding: 10px 12px;
    background: var(--text); color: var(--surface); border-radius: var(--r-card); box-shadow: var(--shadow-lg); font-size: 13px;
  }
  .success :global(svg) { color: var(--pill-ready-dot); }
  .error :global(svg) { color: var(--pill-failed-dot); }
  span { flex: 1; }
  button { border: 0; background: transparent; color: inherit; cursor: pointer; display: flex; padding: 2px; }
  @media (max-width: 767px) { .stack { left: var(--s-4); bottom: calc(var(--tabbar) + var(--s-3)); } }
</style>
