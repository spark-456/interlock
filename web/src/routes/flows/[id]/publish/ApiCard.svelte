<script lang="ts">
  // API card: OpenAI-compatible base URL, model name and a key that is shown once.
  import { api } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import type { Endpoints } from './publish';

  let { endpoints, appName, demo }: { endpoints: Endpoints; appName: string; demo: boolean } = $props();
  let key = $state<string | null>(null);
  let creating = $state(false);

  async function create() {
    creating = true;
    try {
      key = (await api.createKey(appName)).key;
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not create a key.', 'error');
    } finally {
      creating = false;
    }
  }
</script>

<section class="card" aria-labelledby="api-title">
  <header>
    <span class="tile"><Icon name="api" size={18} /></span>
    <div><h2 id="api-title">API</h2><p>OpenAI-compatible · streaming</p></div>
  </header>
  <CopyField label="Base URL" value={endpoints.base} />
  <CopyField label="Model" value={endpoints.model} />
  {#if key}
    <CopyField label="Key" value={key} mode="reveal-once" />
  {:else if !demo}
    <div class="key">
      <span class="label">Key</span>
      <Button icon="plus" loading={creating} onclick={create}>Create a key</Button>
    </div>
  {:else}
    <p class="demo-note">Key creation is unavailable in demo mode.</p>
  {/if}
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-4); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; gap: var(--s-3); align-items: center; }
  .tile { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-card); background: var(--cat-output-tile); color: var(--cat-output-icon); }
  h2 { font: var(--t-h3); margin: 0; }
  header p { margin: 0; color: var(--text-3); font: var(--t-small); }
  .key { display: flex; flex-direction: column; align-items: flex-start; gap: var(--s-2); }
  .label { font: var(--t-small); font-weight: 600; color: var(--text-2); }
  .demo-note { margin: 0; color: var(--text-3); }
</style>
