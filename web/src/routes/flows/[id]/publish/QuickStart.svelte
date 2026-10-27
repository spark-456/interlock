<script lang="ts">
  // Quick start: the same call in cURL, JavaScript, Python, Rust and Claude Code, with copy.
  import CodeTabs from '$lib/design/CodeTabs.svelte';
  import { quickStart, type Endpoints } from './publish';

  let { endpoints, question }: { endpoints: Endpoints; question: string } = $props();
</script>

<section class="card" aria-labelledby="qs-title">
  <h2 id="qs-title">Quick start</h2>
  <p>Works with any OpenAI SDK, LangChain or the Vercel AI SDK. Change the base URL and the model name.</p>
  <CodeTabs snippets={quickStart(endpoints, question)} value="js" />
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-3); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  h2 { font: var(--t-h3); margin: 0; }
  p { margin: 0; color: var(--text-2); }
</style>
