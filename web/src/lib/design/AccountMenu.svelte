<script lang="ts">
  // Account menu at the foot of the sidebar: the person's picture (or initials) and name open
  // Profile, Settings, and Sign in or Sign out. It opens upward; arrow keys move, Escape closes.
  import Icon from './Icon.svelte';
  import { initials } from './format';
  import { menuMove } from './interactions';

  let { name, plan, avatarUrl = null, signedIn, collapsed = false, onsignout }: {
    name: string; plan: string; avatarUrl?: string | null; signedIn: boolean; collapsed?: boolean; onsignout: () => void;
  } = $props();

  type Item = { label: string; icon: string; href?: string; onselect?: () => void };
  const items: Item[] = $derived([
    { label: 'Profile', icon: 'user', href: '/profile' },
    { label: 'Settings', icon: 'sliders', href: '/settings' },
    signedIn ? { label: 'Sign out', icon: 'sign-out', onselect: onsignout } : { label: 'Sign in', icon: 'sign-in', href: '/signin' }
  ]);
  let open = $state(false);
  let active = $state(0);
  let list: HTMLUListElement | undefined = $state();
  let broken = $state(false);
  const id = $props.id();

  function focusItem(i: number) {
    list?.querySelectorAll<HTMLElement>('[role="menuitem"]')[i]?.focus();
  }

  function toggle() {
    open = !open;
    active = 0;
    if (open) queueMicrotask(() => focusItem(0));
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'Escape') return void (open = false);
    const next = menuMove(active, e.key, items.length);
    if (next === null) return;
    e.preventDefault();
    active = next;
    focusItem(next);
  }
</script>

<div class="account" class:collapsed>
  <button type="button" class="trigger" aria-haspopup="menu" aria-expanded={open} aria-controls={id} aria-label="Account: {name}" title={collapsed ? name : undefined} onclick={toggle}>
    {#if avatarUrl && !broken}
      <img class="avatar" src={avatarUrl} alt="" referrerpolicy="no-referrer" onerror={() => (broken = true)} />
    {:else}
      <span class="avatar" aria-hidden="true">{initials(name)}</span>
    {/if}
    {#if !collapsed}
      <span class="who"><span class="name">{name}</span><span class="plan">{plan}</span></span>
      <Icon name="chevron-down" size={16} />
    {/if}
  </button>
  {#if open}
    <ul class="list" role="menu" {id} bind:this={list} onkeydown={key} onfocusout={(e) => !list?.contains(e.relatedTarget as Node) && (open = false)}>
      {#each items as item, i (item.label)}
        <li role="none">
          {#if item.href}
            <a role="menuitem" tabindex={i === active ? 0 : -1} href={item.href} onclick={() => (open = false)}><Icon name={item.icon} size={16} />{item.label}</a>
          {:else}
            <button type="button" role="menuitem" tabindex={i === active ? 0 : -1} onclick={() => ((open = false), item.onselect?.())}><Icon name={item.icon} size={16} />{item.label}</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .account { position: relative; }
  .trigger {
    width: 100%; display: flex; align-items: center; gap: 10px; padding: 4px; border: 0; border-radius: var(--r-control);
    background: none; color: var(--text); font: inherit; text-align: left; cursor: pointer;
  }
  .trigger:hover, .trigger[aria-expanded='true'] { background: var(--surface-2); }
  .trigger :global(svg) { margin-left: auto; color: var(--text-3); }
  .trigger:focus-visible, .list :is(a, button):focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .avatar {
    width: 36px; height: 36px; flex: none; border-radius: 50%; background: var(--primary-soft); color: var(--primary);
    font-weight: 700; font-size: 13px; display: flex; align-items: center; justify-content: center; object-fit: cover;
  }
  .who { display: flex; flex-direction: column; line-height: 18px; min-width: 0; }
  .name { font-weight: 600; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .plan { font-size: 12px; color: var(--text-3); }
  .list {
    position: absolute; bottom: calc(100% + 6px); left: 0; z-index: 30; min-width: 200px; margin: 0; padding: var(--s-1); list-style: none;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-lg);
  }
  .list :is(a, button) {
    width: 100%; display: flex; align-items: center; gap: var(--s-2); padding: var(--s-2) var(--s-3); border: 0; border-radius: var(--r-sm);
    background: none; color: var(--text); font: inherit; font-size: 14px; text-align: left; text-decoration: none; cursor: pointer;
  }
  .list :is(a, button):hover { background: var(--surface-3); }
  .list :global(svg) { color: var(--text-3); }
  .collapsed .trigger { width: auto; padding: 2px; border-radius: 50%; }
</style>
