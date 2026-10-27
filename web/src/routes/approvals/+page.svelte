<script lang="ts">
  // Approvals inbox (mobile-first, WS-4): everything waiting on a person, then recent decisions.
  import { api } from '$lib/api';
  import type { Approval, Decision } from '$lib/api/types';
  import Icon from '$lib/design/Icon.svelte';
  import ApprovalCard from './ApprovalCard.svelte';

  let approvals = $state.raw<Approval[] | null>(null);
  let failed = $state(false);
  let flowNames = $state<Record<string, string>>({});

  api.approvals().then((a) => (approvals = a), () => (failed = true));
  api.flows().then((f) => (flowNames = Object.fromEntries(f.map((x) => [x.id, x.name]))), () => {});

  const pending = $derived(approvals?.filter((a) => a.status === 'pending') ?? []);
  const decided = $derived(
    (approvals?.filter((a) => a.status !== 'pending') ?? []).toSorted((a, b) => (b.decidedAt ?? '').localeCompare(a.decidedAt ?? ''))
  );

  function markDecided(a: Approval, d: Decision) {
    const status = d === 'approve' ? 'approved' : 'rejected';
    approvals = (approvals ?? []).map((x) => (x.id === a.id ? { ...x, status, decidedAt: new Date().toISOString(), decidedBy: 'you' } : x));
  }
</script>

<svelte:head><title>Approvals · Interlock</title></svelte:head>

<div class="page">
  <header>
    <h1>Approvals</h1>
    <p>{pending.length ? `${pending.length} waiting for you.` : 'Nothing is waiting for you.'}</p>
  </header>

  {#if failed}
    <p role="alert" class="error">Could not load approvals. Refresh to try again.</p>
  {:else if approvals}
    <section aria-label="Waiting">
      {#each pending as a (a.id)}
        <ApprovalCard approval={a} flowName={flowNames[a.flowId]} ondecided={(d) => markDecided(a, d)} />
      {:else}
        <div class="empty"><Icon name="inbox" size={28} /><p>When a flow asks before it sends or posts, it shows up here.</p></div>
      {/each}
    </section>
    {#if decided.length}
      <section aria-label="Recent decisions">
        <h2>Recent</h2>
        {#each decided as a (a.id)}
          <ApprovalCard approval={a} flowName={flowNames[a.flowId]} />
        {/each}
      </section>
    {/if}
  {/if}
</div>

<style>
  .page { max-width: 720px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-5); }
  h1 { font: var(--t-h1); margin: 0; }
  h2 { font: var(--t-h3); margin: var(--s-3) 0 0; color: var(--text-2); }
  header p { margin: var(--s-2) 0 0; color: var(--text-2); }
  section { display: flex; flex-direction: column; gap: var(--s-4); }
  .empty {
    display: flex; flex-direction: column; align-items: center; gap: var(--s-2); padding: var(--s-8) var(--s-5); text-align: center;
    color: var(--text-3); border: 1px dashed var(--border-strong); border-radius: var(--r-panel);
  }
  .empty p { margin: 0; }
  .error { color: var(--error-text); }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); gap: var(--s-4); }
  }
</style>
