<script lang="ts">
  // A short label shown on hover and keyboard focus of its child, and read as its description.
  import type { Snippet } from 'svelte';

  let { text, children }: { text: string; children: Snippet } = $props();
  const id = $props.id();
</script>

<span class="wrap" aria-describedby={id}>
  {@render children()}
  <span class="tip" role="tooltip" {id}>{text}</span>
</span>

<style>
  .wrap { position: relative; display: inline-flex; }
  .tip {
    position: absolute; bottom: calc(100% + 6px); left: 50%; transform: translateX(-50%); z-index: 30; pointer-events: none;
    padding: var(--s-1) var(--s-2); border-radius: var(--r-sm); background: var(--text); color: var(--surface);
    font: var(--t-small); white-space: nowrap; opacity: 0; transition: opacity var(--dur-pill) ease;
  }
  .wrap:hover .tip, .wrap:focus-within .tip { opacity: 1; }
</style>
