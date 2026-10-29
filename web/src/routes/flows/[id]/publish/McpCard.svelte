<script lang="ts">
  // MCP card: the app's own MCP server, its tools, and one-click install text for coding tools.
  import AppTile from '$lib/design/AppTile.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import { mcpInstall, type Endpoints } from './publish';

  let { endpoints, tools }: { endpoints: Endpoints; tools: string[] } = $props();
  let copied = $state<string | null>(null);

  async function copy(id: string, text: string) {
    await navigator.clipboard.writeText(text);
    copied = id;
    setTimeout(() => (copied = copied === id ? null : copied), 1500);
  }
</script>

<section class="card" aria-labelledby="mcp-title">
  <header>
    <span class="tile"><Icon name="mcp" size={18} /></span>
    <div><h2 id="mcp-title">MCP server</h2><p>Use it from any MCP client</p></div>
  </header>
  <CopyField label="Server URL" value={endpoints.mcp} />
  <div>
    <span class="label">{tools.length} {tools.length === 1 ? 'tool' : 'tools'}</span>
    <div class="tools">{#each tools as t (t)}<Chip kind="tool">{t}</Chip>{/each}</div>
  </div>
  <ul class="installs">
    {#each mcpInstall(endpoints) as i (i.id)}
      <li>
        <button type="button" onclick={() => copy(i.id, i.text)}>
          <AppTile name={i.label.replace('Add to ', '')} size={22} />
          <span>{i.label}</span>
          <span class="state">{#if copied === i.id}<Icon name="check" size={14} /> Copied{:else}<Icon name="copy" size={14} />{/if}</span>
        </button>
      </li>
    {/each}
  </ul>
</section>

<style>
  .card {
    display: flex; flex-direction: column; min-width: 0; gap: var(--s-4); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; gap: var(--s-3); align-items: center; }
  .tile { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-card); background: var(--cat-kb-tile); color: var(--cat-kb-icon); }
  h2 { font: var(--t-h3); margin: 0; }
  header p { margin: 0; color: var(--text-3); font: var(--t-small); }
  .label { font: var(--t-small); font-weight: 600; color: var(--text-2); }
  .tools { display: flex; flex-wrap: wrap; gap: var(--s-2); margin-top: var(--s-2); }
  .installs { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--s-2); }
  .installs button {
    width: 100%; display: flex; align-items: center; gap: var(--s-3); padding: var(--s-2) var(--s-3); font: inherit; color: var(--text); cursor: pointer;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-button-secondary);
  }
  .installs span:nth-child(2) { flex: 1; text-align: left; font-weight: 600; }
  .state { display: inline-flex; align-items: center; gap: var(--s-1); color: var(--text-3); font: var(--t-small); }
</style>
