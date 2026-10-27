<script lang="ts">
  // Placeholder blocks while content loads; hidden from screen readers.
  let { lines = 1, height = 14, width = '100%' }: { lines?: number; height?: number; width?: string } = $props();
</script>

<span class="skeleton" aria-hidden="true">
  {#each Array(lines) as _, i (i)}
    <span class="line" style:height="{height}px" style:width={i === lines - 1 && lines > 1 ? '60%' : width}></span>
  {/each}
</span>

<style>
  .skeleton { display: flex; flex-direction: column; gap: var(--s-2); width: 100%; }
  .line {
    display: block; border-radius: var(--r-sm);
    background: linear-gradient(90deg, var(--surface-2) 25%, var(--surface-4) 50%, var(--surface-2) 75%); background-size: 200% 100%;
    animation: shimmer 1.4s ease infinite;
  }
  @keyframes shimmer { from { background-position: 200% 0; } to { background-position: -200% 0; } }
  @media (prefers-reduced-motion: reduce) { .line { animation: none; } }
</style>
