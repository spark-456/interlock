<script lang="ts">
  // Partner or tool slot (NVIDIA, OpenAI SDK, Claude Code…): its mark when brands.ts has one, then the name.
  import { brandFor } from './brands';

  let { name, src, height = 20 }: { name: string; src?: string; height?: number } = $props();
  let failed = $state(false);
  const brand = $derived(brandFor(name));
</script>

<span class="slot" style:height="{height}px" title={name}>
  {#if src && !failed}<img {src} alt={name} onerror={() => (failed = true)} />{:else}
    {#if brand}<svg viewBox="0 0 24 24" aria-hidden="true"><path d={brand.path} fill="#{brand.hex}" /></svg>{/if}{name}
  {/if}
</span>

<style>
  .slot {
    flex: 1 1 auto; display: flex; align-items: center; justify-content: center; gap: 6px; padding: 0 8px; border-radius: 5px;
    background: var(--surface-4); font-size: 12px; line-height: 12px; font-weight: 700; color: var(--text-2); white-space: nowrap;
  }
  img { height: 70%; }
  svg { height: 64%; width: auto; flex: none; }
</style>
