<script lang="ts">
  // "Built with" strip: three partner slots. Models are named only here and in Advanced.
  import PartnerSlot from './PartnerSlot.svelte';

  let { partners = [{ name: 'NVIDIA' }, { name: 'Nebius' }, { name: 'Tavily' }] }: { partners?: { name: string; src?: string }[] } = $props();
</script>

<div class="strip">
  <span class="title">Built with</span>
  <div class="slots">
    {#each partners as p (p.name)}<PartnerSlot name={p.name} src={p.src} />{/each}
  </div>
</div>

<style>
  .strip { display: flex; flex-direction: column; gap: 8px; padding: 12px; border: 1px solid var(--border); border-radius: var(--r-card); background: var(--surface-3); }
  .title { font-size: 12px; font-weight: 600; color: var(--text-3); }
  .slots { display: flex; gap: 6px; }
</style>
