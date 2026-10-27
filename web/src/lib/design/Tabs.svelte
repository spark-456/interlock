<script lang="ts">
  // Tabs with arrow-key navigation. The caller renders the panel for `value`.
  type Tab = { value: string; label: string };
  let { tabs, value = $bindable(''), label }: { tabs: Tab[]; value?: string; label: string } = $props();

  function key(e: KeyboardEvent, i: number) {
    const d = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
    if (!d) return;
    const next = tabs[(i + d + tabs.length) % tabs.length];
    if (next) value = next.value;
    const el = (e.currentTarget as HTMLElement).parentElement?.children[(i + d + tabs.length) % tabs.length];
    (el as HTMLElement | undefined)?.focus();
  }
</script>

<div class="tabs" role="tablist" aria-label={label}>
  {#each tabs as t, i (t.value)}
    <button
      type="button"
      role="tab"
      aria-selected={value === t.value}
      tabindex={value === t.value ? 0 : -1}
      onclick={() => (value = t.value)}
      onkeydown={(e) => key(e, i)}>{t.label}</button>
  {/each}
</div>

<style>
  .tabs { display: flex; gap: 20px; border-bottom: 1px solid var(--border); }
  button {
    height: 40px; padding: 0; border: 0; border-bottom: 2px solid transparent; background: transparent;
    color: var(--text-3); font: 600 14px var(--font-sans); cursor: pointer; margin-bottom: -1px;
  }
  button[aria-selected='true'] { color: var(--primary); border-bottom-color: var(--primary); }
  button:hover { color: var(--text); }
  button:focus-visible { outline: none; box-shadow: var(--focus-ring); border-radius: 4px; }
</style>
