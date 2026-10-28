<script lang="ts">
  // App logo tile (Slack, Gmail, Shopify…): white tile with 1px border and the brand's mark in its
  // colour (brands.ts). Pass `src` for an official file; apps without a mark show their first letter.
  import { brandFor } from './brands';

  let { name, src, size = 32 }: { name: string; src?: string; size?: number } = $props();
  let failed = $state(false);
  const brand = $derived(brandFor(name));
</script>

<span class="tile" style:--size="{size}px" title={name}>
  {#if src && !failed}
    <img {src} alt={name} onerror={() => (failed = true)} />
  {:else if brand}
    <svg viewBox="0 0 24 24" role="img" aria-label={name}><path d={brand.path} fill="#{brand.hex}" /></svg>
  {:else}
    <span class="letter" aria-label={name}>{name.slice(0, 1).toUpperCase()}</span>
  {/if}
</span>

<style>
  .tile {
    width: var(--size); height: var(--size); flex: none; display: inline-flex; align-items: center; justify-content: center;
    border-radius: calc(var(--size) * .28); background: var(--cat-app-tile); border: 1px solid var(--cat-app-edge); box-shadow: var(--shadow-tile);
  }
  img, svg { width: 58%; height: 58%; object-fit: contain; }
  .letter { font-weight: 700; font-size: calc(var(--size) * .42); color: var(--cat-app-icon); }
</style>
