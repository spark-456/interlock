<script lang="ts">
  // The same snippet in several languages, with a copy button.
  import Button from './Button.svelte';
  import Tabs from './Tabs.svelte';

  type Snippet = { value: string; label: string; code: string };
  let { snippets, value = $bindable(snippets[0]?.value ?? '') }: { snippets: Snippet[]; value?: string } = $props();
  let copied = $state(false);
  const code = $derived(snippets.find((s) => s.value === value)?.code ?? '');

  async function copy() {
    await navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<div class="codetabs">
  <Tabs label="Language" tabs={snippets} bind:value />
  <div class="code">
    <pre><code>{code}</code></pre>
    <span class="copy"><Button size="s" icon={copied ? 'check' : 'copy'} onclick={copy}>{copied ? 'Copied' : 'Copy'}</Button></span>
  </div>
</div>

<style>
  .codetabs { display: flex; flex-direction: column; gap: var(--s-2); min-width: 0; }
  .code { position: relative; }
  pre { margin: 0; padding: var(--s-4); overflow-x: auto; border-radius: var(--r-card); background: var(--text); color: var(--surface); font: var(--t-code); }
  .copy { position: absolute; top: var(--s-2); right: var(--s-2); }
</style>
