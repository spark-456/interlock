<script lang="ts">
  // How a trigger step starts the flow: its webhook address, where chat/API/MCP calls come in,
  // its schedule in words, or that it's started by hand.
  import { page } from '$app/state';
  import { api } from '$lib/api';
  import type { Flow, Node } from '$lib/api/types';
  import Button from '$lib/design/Button.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import { describeCron } from './triggers';

  let { flow, node, saved }: { flow: Flow; node: Node; saved: boolean } = $props();

  let path = $state<string | null>(null);
  $effect(() => {
    if (node.type !== 'trigger.webhook' || !saved) return;
    api.hooks(flow.id).then((hooks) => (path = hooks.find((h) => h.nodeId === node.id)?.path ?? null), () => {});
  });

  const settings = $derived(node.settings as { cron?: string; timezone?: string; method?: string });
  const schedule = $derived(settings.cron ? (describeCron(settings.cron) ?? `On the schedule ${settings.cron}`) : null);
  const published = $derived(['trigger.chat', 'trigger.api_call', 'trigger.mcp_call'].includes(node.type));
</script>

<section class="info" aria-label="How this step starts the flow">
  {#if node.type === 'trigger.webhook'}
    {#if path}
      <CopyField label="Webhook address" value={page.url.origin + path} />
      <p><Icon name="info" size={14} /> Send a {settings.method ?? 'POST'} request here to start a run. It works while the flow is on.</p>
    {:else}
      <p><Icon name="info" size={14} /> Save the flow to get this step's webhook address.</p>
    {/if}
  {:else if node.type === 'trigger.schedule'}
    <p><Icon name="clock" size={14} /> {schedule ?? 'Set a schedule below.'}{settings.timezone ? ` (${settings.timezone})` : ''}, while the flow is on.</p>
  {:else if published}
    <p><Icon name="info" size={14} /> People reach this through the chat, the API or MCP once you publish the flow.</p>
    <Button size="s" icon="globe" href="/flows/{flow.id}/publish">Open Publish</Button>
  {:else if node.type === 'trigger.manual'}
    <p><Icon name="play" size={14} /> Started by hand: use Test run at the top.</p>
  {:else}
    <p><Icon name="bolt" size={14} /> Starts the flow when this happens, while the flow is on.</p>
  {/if}
</section>

<style>
  .info {
    display: flex; flex-direction: column; align-items: flex-start; gap: var(--s-2); padding: var(--s-3);
    border-radius: var(--r-card); background: var(--surface-3); border: 1px solid var(--border);
  }
  .info > :global(*) { align-self: stretch; }
  .info > :global(.btn) { align-self: flex-start; }
  p { margin: 0; display: flex; gap: var(--s-1); align-items: flex-start; font: var(--t-small); color: var(--text-2); }
  p :global(svg) { flex: none; margin-top: 2px; }
</style>
