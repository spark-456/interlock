<script lang="ts">
  // Quick start: the same call in cURL, JavaScript, Python, Rust and Claude Code, with copy.
  import Button from '$lib/design/Button.svelte';
  import Tabs from '$lib/design/Tabs.svelte';
  import { quickStart, type Endpoints } from './publish';

  let { endpoints, question }: { endpoints: Endpoints; question: string } = $props();
  const snippets = $derived(quickStart(endpoints, question));
  let tab = $state('js');
  let copied = $state(false);
  const code = $derived(snippets.find((s) => s.value === tab)?.code ?? '');

  async function copy() {
    await navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<section class="card" aria-labelledby="qs-title">
  <h2 id="qs-title">Quick start</h2>
  <p>Works with any OpenAI SDK, LangChain or the Vercel AI SDK. Change the base URL and the model name.</p>
  <Tabs label="Language" tabs={snippets} bind:value={tab} />
  <div class="code">
    <pre><code>{code}</code></pre>
    <span class="copy"><Button size="s" icon={copied ? 'check' : 'copy'} onclick={copy}>{copied ? 'Copied' : 'Copy'}</Button></span>
  </div>
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-3); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  h2 { font: var(--t-h3); margin: 0; }
  p { margin: 0; color: var(--text-2); }
  .code { position: relative; }
  pre {
    margin: 0; padding: var(--s-4); overflow-x: auto; border-radius: var(--r-card); background: var(--text); color: var(--surface);
    font: var(--t-small); font-family: var(--font-mono); line-height: 1.7;
  }
  .copy { position: absolute; top: var(--s-2); right: var(--s-2); }
</style>
