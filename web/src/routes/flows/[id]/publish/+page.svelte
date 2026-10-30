<script lang="ts">
  // Publish (design S4 "Connect"): AI apps get API, MCP and widget cards, quick start, the
  // integration pack and the platform MCP; automations get the Activate state.
  import { page } from '$app/state';
  import { api, type Flow } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import PartnerSlot from '$lib/design/PartnerSlot.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import TopBar from '$lib/design/TopBar.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import AgentCard from './AgentCard.svelte';
  import ApiCard from './ApiCard.svelte';
  import GuideCard from './GuideCard.svelte';
  import McpCard from './McpCard.svelte';
  import QuickStart from './QuickStart.svelte';
  import WidgetCard from './WidgetCard.svelte';
  import { endpoints } from './publish';

  const flowId = page.params.id ?? '';
  let flow = $state.raw<Flow | null>(null);
  let active = $state(false);
  let switching = $state(false);
  let failed = $state(false);

  api.flow(flowId).then((f) => (flow = f), () => (failed = true));
  api.flows().then((list) => (active = list.find((f) => f.id === flowId)?.active ?? false), () => {});

  const app = $derived(flow?.kind === 'app');
  const ends = $derived(flow ? endpoints(page.url.origin, flow.name) : null);
  const triggers = $derived(flow?.nodes.filter((n) => n.type.startsWith('trigger.')) ?? []);
  const chat = $derived(flow?.nodes.find((n) => n.type === 'trigger.chat')?.settings as { welcomeMessage?: string } | undefined);
  const WORKS_WITH = ['OpenAI SDK', 'LangChain', 'Vercel AI SDK', 'Claude Code', 'Cursor', 'VS Code'];

  async function setActive(on: boolean) {
    switching = true;
    try {
      await (on ? api.activate(flowId) : api.deactivate(flowId));
      active = on;
      toast(on ? `${flow?.name} is live.` : `${flow?.name} is off.`, 'success');
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not change the flow.', 'error');
    } finally {
      switching = false;
    }
  }
</script>

<svelte:head><title>Publish · {flow?.name ?? 'Interlock'}</title></svelte:head>

<TopBar title={app ? 'Connect' : 'Activate'} crumb={{ href: `/flows/${flowId}`, label: flow?.name ?? 'Flow' }} back="/flows/{flowId}" />

<div class="page">
  {#if failed}
    <p role="alert" class="error">Couldn't open this flow.</p>
  {:else if flow && ends}
    <section class="hero" class:live={active}>
      <span class="mark"><Icon name={active ? 'check' : app ? 'globe' : 'bolt'} size={28} /></span>
      <div class="text">
        <h2 class="title">
          {active ? `${flow.name} is live` : app ? `Publish ${flow.name}` : `Turn on ${flow.name}`}
          <StatusPill tone={active ? 'live' : 'waiting'} label={active ? 'Live' : 'Off'} />
        </h2>
        <p>
          {flow.nodes.length} steps · version {flow.version}
          {#if !app && triggers.length}· starts on {triggers.map((t) => t.name).join(', ')}{/if}
        </p>
      </div>
      <div class="cta">
        {#if !active}
          <Button variant="primary" icon={app ? 'globe' : 'play'} loading={switching} onclick={() => setActive(true)}>{app ? 'Publish' : 'Activate'}</Button>
        {:else if app}
          <Button variant="primary" icon="chat" href="/flows/{flowId}/chat">Open playground</Button>
        {/if}
        {#if active}
          <Button loading={switching} onclick={() => setActive(false)}>{app ? 'Unpublish' : 'Turn off'}</Button>
        {/if}
        <Button href="/flows/{flowId}">View the flow</Button>
      </div>
    </section>

    {#if app}
      <div class="three" data-tour="publish">
        <ApiCard endpoints={ends} appName={flow.name} />
        <McpCard endpoints={ends} tools={[`ask_${ends.model}`]} />
        <WidgetCard endpoints={ends} appName={flow.name} greeting={chat?.welcomeMessage ?? 'Hi! Ask me anything.'} />
      </div>
      <QuickStart endpoints={ends} question="What can you help me with?" />
      <div class="two">
        <GuideCard {flowId} />
        <AgentCard endpoints={ends} appName={flow.name} />
      </div>
      <section class="works" aria-label="Works with">
        <div><h2>Works with</h2><p>Any OpenAI-compatible client or MCP host.</p></div>
        <ul>{#each WORKS_WITH as w (w)}<li><PartnerSlot name={w} height={28} /></li>{/each}</ul>
      </section>
    {:else}
      <section class="note">
        <h2>How it runs</h2>
        <p>
          {active ? 'It runs on its own' : 'Once it’s on, it runs on its own'} whenever {triggers.length ? 'its trigger fires' : 'it’s started'}.
          Every run shows up in the flow’s history, and steps that send or post wait for approval where you asked for it.
        </p>
      </section>
    {/if}
  {/if}
</div>

<style>
  .page { box-sizing: border-box; width: 100%; min-width: 0; max-width: 1160px; margin: 0 auto; padding: var(--s-6) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-5); }
  .hero {
    display: flex; align-items: center; gap: var(--s-5); padding: var(--s-6); flex-wrap: wrap;
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  .mark { display: grid; place-items: center; width: 64px; height: 64px; border-radius: var(--r-panel); background: var(--surface-2); color: var(--text-2); }
  .live .mark { background: var(--success); color: var(--surface); box-shadow: var(--shadow-button-primary); }
  .text { flex: 1; min-width: 240px; }
  .title { font: var(--t-h1); margin: 0; display: flex; align-items: center; gap: var(--s-3); flex-wrap: wrap; }
  .text p { margin: var(--s-2) 0 0; color: var(--text-2); }
  .cta { display: flex; flex-direction: column; gap: var(--s-2); }
  .three { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--s-5); align-items: start; }
  .two { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--s-5); align-items: start; }
  .works, .note {
    display: flex; align-items: center; gap: var(--s-6); padding: var(--s-5); flex-wrap: wrap;
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  .note { flex-direction: column; align-items: flex-start; gap: var(--s-2); }
  h2 { font: var(--t-h3); margin: 0; }
  .works p, .note p { margin: 0; color: var(--text-3); }
  .works ul { flex: 1; list-style: none; margin: 0; padding: 0; display: flex; justify-content: space-between; gap: var(--s-4); flex-wrap: wrap; }
  .works li { display: flex; flex-direction: column; align-items: center; gap: var(--s-1); font: var(--t-small); color: var(--text-2); }
  .error { color: var(--error-text); }
  @media (max-width: 1023px) {
    .three, .two { grid-template-columns: minmax(0, 1fr); }
  }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); }
    .cta { flex-direction: row; flex-wrap: wrap; }
  }
</style>
