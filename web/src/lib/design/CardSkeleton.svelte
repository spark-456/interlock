<script lang="ts">
  // Placeholder cards for a grid that is still loading.
  import Skeleton from './Skeleton.svelte';

  let { count = 3, label }: { count?: number; label: string } = $props();
</script>

<div class="cards" role="status" aria-busy="true" aria-label={label}>
  {#each Array(count) as _, i (i)}
    <div class="card"><Skeleton height={28} width="40%" /><Skeleton lines={3} /></div>
  {/each}
</div>

<style>
  .cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: var(--s-4); }
  .card { display: grid; gap: var(--s-3); padding: var(--s-4); border: 1px solid var(--border); border-radius: var(--r-panel); background: var(--surface); }
</style>
