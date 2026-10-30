<script lang="ts">
  // Sidebar: 240 px, or 64 px icon rail on canvas screens.
  import BuiltWithStrip from './BuiltWithStrip.svelte';
  import Icon from './Icon.svelte';
  import Logo from './Logo.svelte';
  import { initials } from './format';
  import { isActive, sidebarNav } from './nav';

  let { collapsed = false, path, user, plan, onhelp }: {
    collapsed?: boolean; path: string; user: string; plan: string; onhelp?: () => void;
  } = $props();
</script>

<aside class="sidebar" class:collapsed aria-label="Main">
  <a class="brand" href="/" aria-label="Interlock home">
    <Logo variant={collapsed ? 'mark' : 'lockup'} size={28} />
  </a>
  <a class="new" href="/new" aria-label={collapsed ? 'New flow' : undefined} title={collapsed ? 'New flow' : undefined}>
    <Icon name="plus" size={18} strokeWidth={2.4} />
    {#if !collapsed}New flow{/if}
  </a>
  <nav>
    {#each sidebarNav as item (item.href)}
      {@const active = isActive(item.href, path)}
      <a
        href={item.href}
        class:active
        aria-current={active ? 'page' : undefined}
        aria-label={collapsed ? item.label : undefined}
        title={collapsed ? item.label : undefined}>
        <Icon name={item.icon} size={20} />
        {#if !collapsed}<span>{item.label}</span>{/if}
      </a>
    {/each}
  </nav>
  <div class="foot">
    {#if onhelp}
      <button type="button" class="help" data-tour="help" onclick={onhelp} aria-label={collapsed ? 'Help' : undefined} title={collapsed ? 'Help' : undefined}>
        <Icon name="help" size={20} />
        {#if !collapsed}<span>Help</span>{/if}
      </button>
    {/if}
    {#if !collapsed}<BuiltWithStrip />{/if}
    <div class="account">
      <span class="avatar" aria-hidden={!collapsed} title={collapsed ? user : undefined}>{initials(user)}</span>
      {#if !collapsed}
        <span class="who"><span class="name">{user}</span><span class="plan">{plan}</span></span>
      {/if}
    </div>
  </div>
</aside>

<style>
  .sidebar {
    flex: 0 0 var(--sidebar); width: var(--sidebar); height: 100vh; position: sticky; top: 0;
    display: flex; flex-direction: column; gap: 2px; padding: 20px 16px 16px;
    background: var(--surface); border-right: 1px solid var(--border);
  }
  .brand { display: flex; align-items: center; padding: 0 8px; margin-bottom: 20px; color: var(--text); text-decoration: none; }
  .new {
    display: flex; align-items: center; justify-content: center; gap: 8px; height: 40px; border-radius: var(--r-control);
    background: var(--primary); color: var(--surface); font-weight: 600; font-size: 14px; text-decoration: none;
    margin-bottom: 16px; box-shadow: var(--shadow-button-primary);
  }
  .new:hover { background: var(--primary-hover); }
  nav { display: flex; flex-direction: column; gap: 2px; }
  nav a, .help {
    display: flex; align-items: center; gap: 12px; height: 40px; padding: 0 12px; border-radius: var(--r-control);
    color: var(--control-text); font-weight: 600; font-size: 14px; text-decoration: none;
  }
  .help { width: 100%; border: 0; background: none; cursor: pointer; }
  nav a :global(svg), .help :global(svg) { color: var(--text-3); }
  nav a:hover, .help:hover { background: var(--surface-2); }
  nav a.active { background: var(--primary-soft); color: var(--primary); }
  nav a.active :global(svg) { color: var(--primary); }
  .foot { margin-top: auto; display: flex; flex-direction: column; gap: 14px; }
  .collapsed .foot { align-items: center; }
  .account { display: flex; align-items: center; gap: 10px; padding: 0 4px; }
  .avatar {
    width: 36px; height: 36px; flex: none; border-radius: 50%; background: var(--primary-soft); color: var(--primary);
    font-weight: 700; font-size: 13px; display: flex; align-items: center; justify-content: center;
  }
  .who { display: flex; flex-direction: column; line-height: 18px; min-width: 0; }
  .name { font-weight: 600; font-size: 14px; }
  .plan { font-size: 12px; color: var(--text-3); }

  .collapsed { flex-basis: var(--sidebar-collapsed); width: var(--sidebar-collapsed); align-items: center; padding: 16px 0; gap: 4px; }
  .collapsed .brand { width: 40px; height: 40px; padding: 0; justify-content: center; margin-bottom: 12px; }
  .collapsed .new { width: 40px; border-radius: var(--r-tile); margin-bottom: 12px; }
  .collapsed nav { gap: 4px; }
  .collapsed nav a, .collapsed .help { width: 40px; padding: 0; justify-content: center; border-radius: var(--r-tile); }
  .collapsed nav a.active { background: var(--primary-soft); }
  .collapsed .account { padding: 0; }

  @media (max-width: 767px) { .sidebar { display: none; } }
</style>
