<script lang="ts">
  // Mobile bottom tab bar (<768 px), 44 px+ targets.
  import Icon from './Icon.svelte';
  import { isActive, tabNav } from './nav';

  let { path }: { path: string } = $props();
</script>

<nav class="tabbar" aria-label="Tabs">
  {#each tabNav as item (item.href)}
    {@const active = isActive(item.href, path)}
    <a href={item.href} class:active aria-current={active ? 'page' : undefined}>
      <Icon name={item.icon} size={22} />
      {item.label}
    </a>
  {/each}
</nav>

<style>
  .tabbar { display: none; }
  @media (max-width: 767px) {
    .tabbar {
      position: sticky; bottom: 0; z-index: 20; height: var(--tabbar); padding: 0 8px;
      display: grid; grid-template-columns: repeat(4, minmax(0, 1fr));
      background: var(--surface); border-top: 1px solid var(--border);
    }
    a {
      display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px; min-height: 44px;
      color: var(--text-3); font-size: 12px; font-weight: 600; text-decoration: none;
    }
    a.active { color: var(--primary); font-weight: 700; }
  }
</style>
