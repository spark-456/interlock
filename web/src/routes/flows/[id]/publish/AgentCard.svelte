<script lang="ts">
  // "Let your coding agent build the integration": the platform MCP, the command to add it and a prompt.
  import Button from '$lib/design/Button.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import { platformMcpInstall, type Endpoints } from './publish';

  let { endpoints, appName }: { endpoints: Endpoints; appName: string } = $props();
  const TOOLS = ['list_flows', 'get_integration_guide', 'add_documents', 'create_flow_from_goal', 'run_flow'];
  const prompt = $derived(`Add the ${appName} assistant as a chat panel in this app.`);
  let copied = $state(false);

  async function copy() {
    await navigator.clipboard.writeText(prompt);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<section class="card" aria-labelledby="agent-title">
  <header>
    <span class="tile"><Icon name="sparkle" size={18} /></span>
    <div><h2 id="agent-title">Let your coding agent build the integration</h2><p>It fetches the guide and writes the code into your project</p></div>
  </header>
  <CopyField label="Interlock MCP" value={endpoints.platformMcp} />
  <ol>
    <li><CopyField label="Add it to Claude Code" value={platformMcpInstall(endpoints)} /></li>
    <li>
      <p class="prompt">“{prompt}”</p>
      <Button size="s" icon={copied ? 'check' : 'copy'} onclick={copy}>{copied ? 'Copied' : 'Copy prompt'}</Button>
    </li>
  </ol>
  <div class="tools">{#each TOOLS as t (t)}<Chip kind="tool">{t}</Chip>{/each}</div>
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-4); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; gap: var(--s-3); align-items: center; }
  .tile { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-card); background: var(--primary-soft); color: var(--primary); }
  h2 { font: var(--t-h3); margin: 0; }
  header p { margin: 0; color: var(--text-3); font: var(--t-small); }
  ol { margin: 0; padding-left: var(--s-5); display: flex; flex-direction: column; gap: var(--s-3); }
  li::marker { color: var(--primary); font-weight: 600; }
  .prompt { margin: 0 0 var(--s-2); color: var(--text); }
  .tools { display: flex; flex-wrap: wrap; gap: var(--s-2); }
</style>
