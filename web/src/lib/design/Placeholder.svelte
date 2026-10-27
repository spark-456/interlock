<script lang="ts">
  // Placeholder screen used by routes until their owning task builds them.
  import type { Snippet } from 'svelte';
  import TopBar from './TopBar.svelte';

  let { title, task, route, children }: { title: string; task: string; route: string; children?: Snippet } = $props();
</script>

<TopBar {title} />
<section class="placeholder" data-testid="placeholder">
  <div class="card">
    <h2>{title}</h2>
    <p>Route <code>{route}</code>. This screen is built in task <strong>{task}</strong>.</p>
    {@render children?.()}
  </div>
</section>

<style>
  .placeholder { padding: var(--s-8) var(--s-6); }
  .card {
    max-width: 720px; display: grid; gap: var(--s-3); padding: var(--s-6); border-radius: var(--r-card);
    background: var(--card-gradient); border: 1px solid var(--border); box-shadow: var(--shadow-pop);
  }
  h2 { font: var(--t-h2); }
  p { color: var(--text-2); margin: 0; }
  @media (max-width: 767px) { .placeholder { padding: var(--s-4); } }
</style>
