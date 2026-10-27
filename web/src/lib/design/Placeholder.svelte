<script lang="ts">
  // Placeholder screen used by T00 routes until their owning task builds them.
  import type { Snippet } from 'svelte';

  let { title, task, route, children }: { title: string; task: string; route: string; children?: Snippet } = $props();
</script>

<section class="placeholder" data-testid="placeholder">
  <h1>{title}</h1>
  <p>Route <code>{route}</code> · owned by task <strong>{task}</strong></p>
  {@render children?.()}
</section>

<style>
  .placeholder { display: grid; gap: var(--s-3); max-width: 720px; }
  p { color: var(--text-2); margin: 0; }
</style>
