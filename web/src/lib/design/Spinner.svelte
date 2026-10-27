<script lang="ts">
  // The design's two-tone spinner (track + rotating arc).
  let { size = 12, label }: { size?: number; label?: string } = $props();
</script>

<svg class="spinner" width={size} height={size} viewBox="0 0 12 12" role={label ? 'img' : undefined} aria-label={label} aria-hidden={label ? undefined : 'true'}>
  <circle cx="6" cy="6" r="4.5" fill="none" class="track" stroke-width="1.75" />
  <path d="M6 1.5a4.5 4.5 0 0 1 4.5 4.5" fill="none" class="arc" stroke-width="1.75" stroke-linecap="round" />
</svg>

<style>
  .spinner { flex: none; }
  .track { stroke: var(--primary-line); }
  .arc { stroke: currentColor; transform-origin: 6px 6px; animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .arc { animation-duration: 2.4s; } }
</style>
