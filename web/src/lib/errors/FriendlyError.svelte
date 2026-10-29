<script lang="ts">
  // A step or run error in plain words, with the next step to take and the original text on demand.
  import Button from '$lib/design/Button.svelte';
  import { friendlyError } from './friendly';

  let { error }: { error: string } = $props();
  const f = $derived(friendlyError(error));
</script>

<div class="friendly" role="alert">
  <p>{f.message}</p>
  {#if f.action}<Button size="s" href={f.action.href}>{f.action.label}</Button>{/if}
  {#if f.message !== error}
    <details>
      <summary>Details</summary>
      <code>{error}</code>
    </details>
  {/if}
</div>

<style>
  .friendly { display: grid; gap: var(--s-2); justify-items: start; padding: var(--s-2) var(--s-3); border-radius: var(--r-control); background: var(--error-soft); color: var(--error-text); font-size: 13px; }
  p { margin: 0; }
  details { font-size: 12px; }
  summary { cursor: pointer; }
  code { display: block; margin-top: 4px; white-space: pre-wrap; word-break: break-word; font-family: var(--font-mono); }
</style>
