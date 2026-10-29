<script lang="ts">
  // Flows list (design S7 "Your backends"): automations and AI apps together, filtered by status and search.
  import { api, appNames as loadAppNames } from '$lib/api';
  import type { FlowSummary } from '$lib/api/types';
  import Button from '$lib/design/Button.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import Input from '$lib/design/Input.svelte';
  import Logo from '$lib/design/Logo.svelte';
  import Modal from '$lib/design/Modal.svelte';
  import Skeleton from '$lib/design/Skeleton.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import FlowCard from './FlowCard.svelte';
  import { filterFlows, type Filter } from './flows';

  let flows = $state.raw<FlowSummary[] | null>(null);
  let failed = $state(false);
  let appNames = $state<Record<string, string>>({});
  let filter = $state<Filter>('all');
  let query = $state('');
  const now = new Date();

  api.flows().then((f) => (flows = f), () => (failed = true));

  let deleting = $state.raw<FlowSummary | null>(null);
  let confirming = $state(false);
  async function remove() {
    const f = deleting;
    confirming = false;
    if (!f) return;
    try {
      await api.deleteFlow(f.id);
      flows = flows?.filter((x) => x.id !== f.id) ?? null;
      toast(`Deleted “${f.name}”.`);
    } catch (e) {
      toast((e as Error).message, 'error');
    }
  }
  loadAppNames().then((names) => (appNames = names), () => {});

  const live = $derived(flows?.filter((f) => f.active).length ?? 0);
  const shown = $derived(flows ? filterFlows(flows, filter, query) : []);
  const chips: { id: Filter; label: string; count: () => number }[] = [
    { id: 'all', label: 'All', count: () => flows?.length ?? 0 },
    { id: 'live', label: 'Live', count: () => live },
    { id: 'off', label: 'Off', count: () => (flows?.length ?? 0) - live }
  ];
</script>

<svelte:head><title>Flows · Interlock</title></svelte:head>

<div class="page">
  <header>
    <div>
      <h1>Flows</h1>
      {#if flows}
        <p>
          {flows.length} {flows.length === 1 ? 'flow' : 'flows'}. {live} live.
          {#if live < flows.length}New flows start off: try one with Test run, which never posts or sends, then turn it on.{/if}
        </p>
      {/if}
    </div>
    <div class="actions">
      <Input type="search" placeholder="Search flows" aria-label="Search flows" bind:value={query} />
      <Button variant="primary" icon="plus" href="/new">New flow</Button>
    </div>
  </header>

  {#if failed}
    <p role="alert" class="error">Could not load your flows. Refresh to try again.</p>
  {:else if flows}
    {#if flows.length}
      <div class="chips" role="group" aria-label="Filter flows">
        {#each chips as c (c.id)}
          <Chip kind="filter" selected={filter === c.id} onclick={() => (filter = c.id)}>{c.label} <span class="count">{c.count()}</span></Chip>
        {/each}
      </div>
    {/if}
    <div class="grid">
      {#each shown as f (f.id)}
        <FlowCard flow={f} {appNames} {now} ondelete={() => ((deleting = f), (confirming = true))} />
      {/each}
      {#if !flows.length}
        <section class="empty">
          <Logo size={48} />
          <h2>Your first flow is two minutes away.</h2>
          <p>Describe a goal, drop in a few documents, and Interlock does the rest.</p>
          <div class="row">
            <Button variant="primary" href="/new">New flow</Button>
            <Button href="/demo">Try a demo</Button>
          </div>
        </section>
      {:else if !shown.length}
        <p class="none">No flows match.</p>
      {/if}
    </div>
  {:else}
    <div class="grid" role="status" aria-busy="true" aria-label="Loading flows">
      {#each [0, 1, 2] as i (i)}<div class="placeholder"><Skeleton height={112} /><Skeleton lines={3} /></div>{/each}
    </div>
  {/if}
</div>

<Modal bind:open={confirming} title="Delete this flow?">
  <p>“{deleting?.name}” and its run history will be deleted. This can't be undone.</p>
  {#snippet footer()}
    <Button onclick={() => (confirming = false)}>Keep it</Button>
    <Button variant="danger" onclick={remove}>Delete flow</Button>
  {/snippet}
</Modal>

<style>
  .placeholder { display: grid; gap: var(--s-3); padding: var(--s-4); border: 1px solid var(--border); border-radius: var(--r-panel); }
  .page { max-width: 1120px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-5); }
  header { display: flex; justify-content: space-between; align-items: flex-end; gap: var(--s-4); flex-wrap: wrap; }
  h1 { font: var(--t-h1); margin: 0; }
  header p { margin: var(--s-2) 0 0; color: var(--text-2); }
  .actions { display: flex; gap: var(--s-3); align-items: center; }
  .chips { display: flex; gap: var(--s-2); flex-wrap: wrap; }
  .count { color: var(--text-3); margin-left: var(--s-1); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: var(--s-5); }
  .empty {
    display: flex; flex-direction: column; align-items: center; text-align: center; gap: var(--s-3); padding: var(--s-8) var(--s-6);
    border: 1px dashed var(--border-strong); border-radius: var(--r-panel);
  }
  .empty h2 { font: var(--t-h3); margin: 0; }
  .empty p { margin: 0; color: var(--text-2); }
  .row { display: flex; gap: var(--s-2); }
  .none, .error { color: var(--text-2); }
  .error { color: var(--error-text); }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); }
    .actions { width: 100%; }
    .actions :global(.field) { flex: 1; }
  }
</style>
