<script lang="ts">
  // A button that opens a list of actions. Arrow keys move, Enter picks, Escape closes.
  import Icon from './Icon.svelte';
  import { menuMove } from './interactions';

  type Item = { label: string; icon?: string; danger?: boolean; onselect: () => void };
  let { label, icon = 'menu', items }: { label: string; icon?: string; items: Item[] } = $props();
  let open = $state(false);
  let active = $state(0);
  let list: HTMLUListElement | undefined = $state();
  const id = $props.id();

  function show() {
    open = true;
    active = 0;
    queueMicrotask(() => list?.querySelectorAll<HTMLElement>('[role="menuitem"]')[0]?.focus());
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'Escape') return void (open = false);
    const next = menuMove(active, e.key, items.length);
    if (next === null) return;
    e.preventDefault();
    active = next;
    list?.querySelectorAll<HTMLElement>('[role="menuitem"]')[next]?.focus();
  }

  function pick(item: Item) {
    open = false;
    item.onselect();
  }
</script>

<div class="menu">
  <button type="button" class="trigger" aria-haspopup="menu" aria-expanded={open} aria-controls={id} aria-label={label} title={label} onclick={() => (open ? (open = false) : show())}>
    <Icon name={icon} size={16} />
  </button>
  {#if open}
    <ul class="list" role="menu" {id} bind:this={list} onkeydown={key} onfocusout={(e) => !list?.contains(e.relatedTarget as Node) && (open = false)}>
      {#each items as item, i (item.label)}
        <li role="none">
          <button type="button" role="menuitem" tabindex={i === active ? 0 : -1} class:danger={item.danger} onclick={() => pick(item)}>
            {#if item.icon}<Icon name={item.icon} size={14} />{/if}{item.label}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .menu { position: relative; display: inline-flex; }
  .trigger {
    display: grid; place-items: center; width: 32px; height: 32px; border: 0; border-radius: var(--r-md); background: none; color: var(--text-2); cursor: pointer;
  }
  .trigger:hover, .trigger[aria-expanded='true'] { background: var(--surface-3); }
  .trigger:focus-visible, .list button:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .list {
    position: absolute; top: calc(100% + 4px); right: 0; z-index: 30; min-width: 180px; margin: 0; padding: var(--s-1); list-style: none;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-lg);
  }
  .list button {
    width: 100%; display: flex; align-items: center; gap: var(--s-2); padding: var(--s-2) var(--s-3); border: 0; border-radius: var(--r-sm);
    background: none; color: var(--text); font: inherit; text-align: left; cursor: pointer;
  }
  .list button:hover { background: var(--surface-3); }
  .list button.danger { color: var(--error-text); }
</style>
