<script lang="ts">
  // Integration pack: a preview of the app's INTERLOCK.md with copy and download.
  import { api } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';

  let { flowId }: { flowId: string } = $props();
  let markdown = $state<string | null>(null);
  let failed = $state(false);
  let copied = $state(false);

  $effect(() => {
    api.guide(flowId).then((g) => (markdown = g), () => (failed = true));
  });

  const size = $derived(markdown ? `${(new TextEncoder().encode(markdown).length / 1024).toFixed(1)} KB` : '');
  const download = $derived(markdown ? `data:text/markdown;charset=utf-8,${encodeURIComponent(markdown)}` : undefined);

  async function copy() {
    if (!markdown) return;
    await navigator.clipboard.writeText(markdown);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<section class="card" aria-labelledby="guide-title">
  <header>
    <span class="tile"><Icon name="docs" size={18} /></span>
    <div><h2 id="guide-title">Integration pack</h2><p><code>INTERLOCK.md</code>{#if size} · {size}{/if} · endpoints, auth, examples</p></div>
  </header>
  {#if markdown}
    <pre aria-label="INTERLOCK.md preview">{markdown}</pre>
    <div class="actions">
      <Button icon={copied ? 'check' : 'copy'} onclick={copy}>{copied ? 'Copied' : 'Copy for your coding agent'}</Button>
      <a class="download" href={download} download="INTERLOCK.md"><Icon name="upload" size={14} /> Download</a>
    </div>
  {:else if failed}
    <p class="muted">The guide appears here once the app is published.</p>
  {/if}
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-4); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; gap: var(--s-3); align-items: center; }
  .tile { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-card); background: var(--surface-2); color: var(--text-2); }
  h2 { font: var(--t-h3); margin: 0; }
  header p, .muted { margin: 0; color: var(--text-3); font: var(--t-small); }
  pre {
    margin: 0; max-height: 260px; overflow: auto; padding: var(--s-3) var(--s-4); border: 1px solid var(--border); border-radius: var(--r-card);
    background: var(--surface-3); font: var(--t-small); font-family: var(--font-mono); white-space: pre-wrap;
  }
  .actions { display: flex; align-items: center; gap: var(--s-4); flex-wrap: wrap; }
  .download { display: inline-flex; align-items: center; gap: var(--s-1); color: var(--primary); font-weight: 600; text-decoration: none; }
  .download :global(svg) { transform: rotate(180deg); }
</style>
