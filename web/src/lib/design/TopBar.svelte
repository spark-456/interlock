<script lang="ts">
  // Top bar: 64 px. Desktop: breadcrumb + title + status + actions. Mobile (<768): back + title + subtitle + menu.
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    title,
    crumb,
    subtitle,
    back,
    status,
    actions,
    onmenu
  }: {
    title: string;
    crumb?: { href: string; label: string };
    subtitle?: string;
    back?: string;
    status?: Snippet;
    actions?: Snippet;
    onmenu?: () => void;
  } = $props();
</script>

<header class="topbar">
  <div class="desk">
    {#if crumb}
      <a class="crumb" href={crumb.href}>{crumb.label}</a>
      <Icon name="chevron-right" size={16} color="var(--text-4)" />
    {/if}
    <h1>{title}</h1>
    {@render status?.()}
    {#if actions}<div class="actions">{@render actions()}</div>{/if}
  </div>
  <div class="mob">
    {#if back}<a class="tap" href={back} aria-label="Back"><Icon name="chevron-left" size={22} /></a>{/if}
    <div class="titles">
      <span class="mt">{title.replace(/^Building /, '')}</span>
      {#if subtitle}<span class="ms">{subtitle}</span>{/if}
    </div>
    {#if onmenu}<button class="tap" type="button" aria-label="Open menu" onclick={onmenu}><Icon name="menu" size={22} /></button>{/if}
  </div>
</header>

<style>
  .topbar {
    flex: 0 0 var(--topbar); height: var(--topbar); display: flex; align-items: center;
    background: var(--surface); border-bottom: 1px solid var(--border);
  }
  .desk { flex: 1; display: flex; align-items: center; gap: 14px; padding: 0 20px 0 24px; min-width: 0; }
  .crumb { font-size: 14px; font-weight: 600; color: var(--text-3); text-decoration: none; }
  .crumb:hover { color: var(--text); }
  h1 { font-size: 18px; line-height: 26px; font-weight: 700; letter-spacing: -0.01em; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .actions { margin-left: auto; display: flex; align-items: center; gap: 8px; }
  .mob { display: none; }
  @media (max-width: 767px) {
    .topbar { flex-basis: 60px; height: 60px; padding: 0 8px; }
    .desk { display: none; }
    .mob { flex: 1; display: flex; align-items: center; gap: 4px; }
    .titles { flex: 1; min-width: 0; display: flex; flex-direction: column; line-height: 18px; }
    .mt { font-size: 16px; font-weight: 700; }
    .ms { font-size: 12px; color: var(--text-3); }
    .tap {
      width: 44px; height: 44px; border-radius: var(--r-tile); border: 0; background: transparent; color: var(--text);
      display: flex; align-items: center; justify-content: center; cursor: pointer;
    }
  }
</style>
