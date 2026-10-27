<script lang="ts">
  // App logo slot (Slack, Gmail, Shopify…): white tile with 1px border. Pass `src` for the
  // official SVG; otherwise the app name's first letter is shown. Never draw third-party logos.
  let { name, src, size = 32 }: { name: string; src?: string; size?: number } = $props();
  let failed = $state(false);
</script>

<span class="tile" style:--size="{size}px" title={name}>
  {#if src && !failed}
    <img {src} alt={name} onerror={() => (failed = true)} />
  {:else}
    <span class="letter" aria-label={name}>{name.slice(0, 1).toUpperCase()}</span>
  {/if}
</span>

<style>
  .tile {
    width: var(--size); height: var(--size); flex: none; display: inline-flex; align-items: center; justify-content: center;
    border-radius: calc(var(--size) * .28); background: var(--cat-app-tile); border: 1px solid var(--cat-app-edge); box-shadow: var(--shadow-tile);
  }
  img { width: 62%; height: 62%; object-fit: contain; }
  .letter { font-weight: 700; font-size: calc(var(--size) * .42); color: var(--cat-app-icon); }
</style>
