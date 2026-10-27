<script lang="ts">
  // T00 shell: replaced by the design-system shell (W01).
  import '../app.css';
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import { isFixtureMode } from '$lib/api';

  let { children }: { children: Snippet } = $props();

  const nav = [
    { href: '/new', label: 'New' },
    { href: '/flows', label: 'Flows' },
    { href: '/templates', label: 'Templates' },
    { href: '/approvals', label: 'Approvals' },
    { href: '/connections', label: 'Connections' },
    { href: '/settings', label: 'Settings' },
    { href: '/demo', label: 'Demo' }
  ];
</script>

<div class="shell">
  <nav aria-label="Main">
    <a class="brand" href="/">Interlock</a>
    {#each nav as item (item.href)}
      <a href={item.href} aria-current={page.url.pathname.startsWith(item.href) ? 'page' : undefined}>{item.label}</a>
    {/each}
    {#if isFixtureMode()}<span class="fixture">Fixture mode</span>{/if}
  </nav>
  <main>{@render children()}</main>
</div>

<style>
  .shell { display: grid; grid-template-columns: 240px 1fr; min-height: 100vh; }
  nav { display: flex; flex-direction: column; gap: var(--s-1); padding: var(--s-4); background: var(--surface); border-right: 1px solid var(--border); }
  nav a { padding: var(--s-2) var(--s-3); border-radius: var(--r-md); color: var(--text-2); text-decoration: none; font: var(--t-small); }
  nav a[aria-current='page'] { background: var(--primary-soft); color: var(--primary); }
  .brand { font: var(--t-h3); color: var(--text); margin-bottom: var(--s-4); }
  .fixture { margin-top: auto; font: var(--t-caption); color: var(--text-3); }
  main { padding: var(--s-8); }
  @media (max-width: 767px) { .shell { grid-template-columns: 1fr; } nav { flex-direction: row; overflow-x: auto; } }
</style>
