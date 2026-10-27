<script lang="ts">
  // Partner logo slot (NVIDIA, Nebius, Tavily). Optional official SVG via `src`; falls back to the name.
  let { name, src, height = 20 }: { name: string; src?: string; height?: number } = $props();
  let failed = $state(false);
</script>

<span class="slot" style:height="{height}px" title={name}>
  {#if src && !failed}<img {src} alt={name} onerror={() => (failed = true)} />{:else}{name}{/if}
</span>

<style>
  .slot {
    flex: 1 1 auto; display: flex; align-items: center; justify-content: center; padding: 0 6px; border-radius: 5px;
    background: var(--surface-4); font-size: 12px; line-height: 12px; font-weight: 700; color: var(--text-2); white-space: nowrap;
  }
  img { height: 70%; }
</style>
