<script lang="ts">
  // Export as code (Q36): the flow as one folder that runs on the open-source engine in the user's own
  // infrastructure, downloaded here or pulled into a repository by a coding agent.
  import { api } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import { exportCommand, slug } from './publish';

  let { flowId, name, origin }: { flowId: string; name: string; origin: string } = $props();
  const INSIDE = ['flows/', 'prompts/', 'files/', 'compose.yaml', '.env.example', 'README.md', 'INTERLOCK.md'];
</script>

<section class="card" aria-labelledby="export-title">
  <header>
    <span class="tile"><Icon name="box" size={18} /></span>
    <div class="text">
      <h2 id="export-title">Run it in your own infrastructure</h2>
      <p>Export as code: a folder that runs with the open-source engine and <code>docker compose up</code>, with the same API, MCP and webhooks. Keys stay out; you add your own.</p>
    </div>
    <Button variant="primary" href={api.exportHref(flowId)} download="{slug(name)}-interlock.zip">Export as code</Button>
  </header>
  <CopyField label="Or let your coding agent pull it into your repository" value={exportCommand(origin, flowId, name)} />
  <p class="inside">Inside: {#each INSIDE as f, i (f)}<code>{f}</code>{i < INSIDE.length - 1 ? ' · ' : ''}{/each}</p>
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-4); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; gap: var(--s-3); align-items: center; flex-wrap: wrap; }
  .text { flex: 1; min-width: 220px; }
  .tile { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-card); background: var(--surface-2); color: var(--text-2); }
  h2 { font: var(--t-h3); margin: 0; }
  header p, .inside { margin: 0; color: var(--text-3); font: var(--t-small); }
  code { font-family: var(--font-mono); }
</style>
