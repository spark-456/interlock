<script lang="ts">
  // 404 and unexpected errors, inside the app shell.
  import { page } from '$app/state';
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';

  const missing = $derived(page.status === 404);
</script>

<svelte:head><title>{missing ? 'Not found' : 'Something went wrong'} · Interlock</title></svelte:head>

<section class="state" role="alert">
  <span class="mark"><Icon name={missing ? 'search' : 'alert'} size={28} /></span>
  <h1>{missing ? 'This page doesn’t exist' : 'Something went wrong'}</h1>
  <p>
    {#if missing}
      The link may be old, or the flow was deleted.
    {:else}
      {page.error?.message ?? 'An unexpected error stopped this page.'} Try again; if it keeps happening, start from your flows.
    {/if}
  </p>
  <div class="actions">
    <Button variant="primary" href="/flows">Go to your flows</Button>
    {#if !missing}<Button onclick={() => location.reload()}>Try again</Button>{/if}
  </div>
  <span class="code">{page.status}</span>
</section>

<style>
  .state { margin: auto; max-width: 440px; padding: var(--s-10) var(--s-6); display: flex; flex-direction: column; align-items: center; gap: var(--s-3); text-align: center; }
  .mark { display: grid; place-items: center; width: 56px; height: 56px; border-radius: var(--r-panel); background: var(--surface-2); color: var(--text-2); }
  h1 { font: var(--t-h1); margin: 0; }
  p { margin: 0; color: var(--text-2); }
  .actions { display: flex; gap: var(--s-2); margin-top: var(--s-2); }
  .code { color: var(--text-4); font: var(--t-small); font-family: var(--font-mono); }
</style>
