<script lang="ts">
  // Settings: the workspace you're in, API keys (create once, revoke), how spending is capped, and
  // deleting your account.
  import { api, type Me, type UsageCaps, type UsageReport } from '$lib/api';
  import { usd } from '$lib/design/format';
  import Button from '$lib/design/Button.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import Input from '$lib/design/Input.svelte';
  import Modal from '$lib/design/Modal.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import { ago } from '../flows/flows';
  import Skeleton from '$lib/design/Skeleton.svelte';
  import Checklist from '$lib/onboarding/Checklist.svelte';
  import { onboarding, startTour } from '$lib/onboarding/state.svelte';

  type Key = Awaited<ReturnType<typeof api.keys>>[number];
  let me = $state.raw<Me | null>(null);
  let keys = $state.raw<Key[] | null>(null);
  let keysError = $state<string | null>(null);
  let name = $state('');
  let created = $state<string | null>(null);
  let busy = $state<string | null>(null);
  let confirmingDelete = $state(false);
  let deleteText = $state('');
  const now = new Date();

  api.me().then((m) => (me = m), () => {});
  let usage = $state.raw<UsageReport | null>(null);
  let runCap = $state('');
  let flowCap = $state('');
  let workspaceCap = $state('');
  api.usage().then((u) => {
    usage = u;
    runCap = String(u.caps.runUsd);
    flowCap = String(u.caps.flowDailyUsd);
    workspaceCap = String(u.caps.workspaceDailyUsd);
  }, () => {});
  const barHeight = (cost: number) => {
    const max = Math.max(...(usage?.days.map((d) => d.costUsd) ?? [0]), 0.000001);
    return cost > 0 ? Math.max(2, (cost / max) * 58) : 1;
  };
  const loadKeys = () => api.keys().then((k) => (keys = k), (e) => (keysError = e instanceof Error ? e.message : 'Could not load keys.'));
  loadKeys();

  async function create() {
    busy = 'create';
    try {
      created = (await api.createKey(name.trim() || 'API key')).key;
      name = '';
      await loadKeys();
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not create a key.', 'error');
    } finally {
      busy = null;
    }
  }

  async function revoke(k: Key) {
    busy = k.id;
    try {
      await api.revokeKey(k.id);
      keys = (keys ?? []).map((x) => (x.id === k.id ? { ...x, revoked: true } : x));
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not revoke the key.', 'error');
    } finally {
      busy = null;
    }
  }

  async function saveCaps() {
    const caps: UsageCaps = {
      runUsd: Number(runCap), flowDailyUsd: Number(flowCap), workspaceDailyUsd: Number(workspaceCap)
    };
    if (!Number.isFinite(caps.runUsd) || caps.runUsd <= 0 ||
        !Number.isFinite(caps.flowDailyUsd) || caps.flowDailyUsd < caps.runUsd ||
        !Number.isFinite(caps.workspaceDailyUsd) || caps.workspaceDailyUsd < caps.flowDailyUsd) {
      toast('Enter positive caps ordered per run, flow, then workspace.', 'error');
      return;
    }
    busy = 'caps';
    try {
      const saved = await api.updateUsageCaps(caps);
      if (usage) usage = { ...usage, caps: saved };
      toast('Workspace caps saved.', 'success');
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not save workspace caps.', 'error');
    } finally {
      busy = null;
    }
  }

  async function signOut() {
    busy = 'signout';
    try {
      await api.signOut();
      location.assign('/');
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not sign out.', 'error');
      busy = null;
    }
  }

  async function deleteAccount() {
    busy = 'delete';
    try {
      await api.deleteAccount();
      location.assign('/');
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not delete the account.', 'error');
      busy = null;
    }
  }
</script>

<svelte:head><title>Settings · Interlock</title></svelte:head>

<div class="page">
  <h1>Settings</h1>

  <section aria-labelledby="ws">
    <h2 id="ws">Workspace</h2>
    {#if me}
      <dl>
        <div><dt>Signed in as</dt><dd>{me.user?.name ?? 'Guest (this computer only)'}</dd></div>
        <div><dt>Workspace</dt><dd class="mono">{me.workspaceId}</dd></div>
        <div><dt>Plan</dt><dd>Free plan</dd></div>
      </dl>
      {#if me.user}
        <Button loading={busy === 'signout'} onclick={signOut}>Sign out</Button>
      {:else}
        <Button variant="primary" href="/signin">Sign in</Button>
      {/if}
    {:else}
      <Skeleton lines={3} />
    {/if}
  </section>

  <section aria-labelledby="keys">
    <h2 id="keys">API keys</h2>
    <p class="muted">Keys call your published apps over the OpenAI-compatible API and MCP. A key is shown once, when you create it.</p>
    {#if keysError}
      <p class="muted">{keysError}</p>
    {:else}
      <form class="create" onsubmit={(e) => (e.preventDefault(), create())}>
        <Input label="New key name" placeholder="Website widget" bind:value={name} />
        <Button type="submit" icon="plus" loading={busy === 'create'}>Create key</Button>
      </form>
      {#if created}<CopyField label="Your new key" value={created} mode="reveal-once" />{/if}
      <ul class="keys" aria-label="API keys">
        {#each keys ?? [] as k (k.id)}
          <li>
            <span class="name">{k.name}</span>
            <span class="muted">{k.lastUsedAt ? `Used ${ago(k.lastUsedAt, now)}` : 'Never used'}</span>
            {#if k.revoked}
              <StatusPill tone="waiting" label="Revoked" />
            {:else}
              <Button size="s" variant="danger" loading={busy === k.id} disabled={!!busy} onclick={() => revoke(k)}>Revoke</Button>
            {/if}
          </li>
        {:else}
          <li class="muted">No keys yet.</li>
        {/each}
      </ul>
    {/if}
  </section>

  <section aria-labelledby="usage">
    <h2 id="usage">Usage and caps</h2>
    {#if usage}
      <p class="total"><span class="mono">{usd(usage.totalUsd)}</span> spent in the last 30 days</p>
      <svg class="bars" viewBox="0 0 300 60" role="img" aria-label="Spend per day for the last 30 days">
        {#each usage.days as d, i (d.day)}
          <rect x={i * 10 + 1} y={60 - barHeight(d.costUsd)} width="8" height={barHeight(d.costUsd)} rx="1"><title>{d.day}: {usd(d.costUsd)}</title></rect>
        {/each}
      </svg>
      {#if usage.flows.length}
        <ul class="flows" aria-label="Spend by flow">
          {#each usage.flows as f (f.flowId)}<li><a href="/flows/{f.flowId}">{f.name}</a><span class="mono">{usd(f.costUsd)}</span></li>{/each}
        </ul>
      {/if}
      {#if usage.roles.length}
        <h3>Spend by role</h3>
        <ul class="roles" aria-label="Spend by model role">
          {#each usage.roles as role (role.role)}<li><span>{role.role}</span><span class="mono">{usd(role.costUsd)}</span></li>{/each}
        </ul>
      {/if}
      <dl class="caps">
        <div><dt>Per run</dt><dd class="mono">{usd(usage.caps.runUsd)}</dd></div>
        <div><dt>Per flow, per day</dt><dd class="mono">{usd(usage.caps.flowDailyUsd)}</dd></div>
        <div><dt>Workspace, per day</dt><dd class="mono">{usd(usage.caps.workspaceDailyUsd)}</dd></div>
      </dl>
      <form class="cap-form" onsubmit={(e) => (e.preventDefault(), saveCaps())}>
        <Input label="Per run (USD)" type="number" min="0.01" step="0.01" bind:value={runCap} />
        <Input label="Per flow, per day (USD)" type="number" min="0.01" step="0.01" bind:value={flowCap} />
        <Input label="Workspace, per day (USD)" type="number" min="0.01" step="0.01" bind:value={workspaceCap} />
        <Button type="submit" variant="primary" loading={busy === 'caps'}>Save caps</Button>
      </form>
    {/if}
    <p class="muted">
      When a cap is reached the run stops and says so. A flow can set its own lower per-run cap in its settings.
    </p>
  </section>

  <section aria-labelledby="start">
    <h2 id="start">Getting started</h2>
    <p class="muted">The tour points at each screen in about a minute. The checklist ticks itself off as you go.</p>
    <Checklist />
    <div class="row">
      <Button variant="primary" icon="play" onclick={startTour}>Take the tour</Button>
      <Button onclick={() => (onboarding.welcomeOpen = true)}>Show the welcome</Button>
    </div>
  </section>

  {#if me?.user}
    <section aria-labelledby="account">
      <h2 id="account">Delete account</h2>
      <p class="muted">Turns off your flows and permanently deletes them, with their runs, knowledge bases, files, connections and API keys. This can't be undone.</p>
      <Button variant="danger" onclick={() => ((deleteText = ''), (confirmingDelete = true))}>Delete my account</Button>
    </section>
  {/if}
</div>

<Modal bind:open={confirmingDelete} title="Delete your account?">
  <div class="confirm">
    <p class="muted">Everything in your workspace is deleted for good. Type DELETE to confirm.</p>
    <Input label="Type DELETE" bind:value={deleteText} />
  </div>
  {#snippet footer()}
    <Button onclick={() => (confirmingDelete = false)}>Cancel</Button>
    <Button variant="danger" loading={busy === 'delete'} disabled={deleteText !== 'DELETE'} onclick={deleteAccount}>Delete everything</Button>
  {/snippet}
</Modal>

<style>
  .page { max-width: 760px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-5); }
  h1 { font: var(--t-h1); margin: 0; }
  section {
    display: flex; flex-direction: column; align-items: flex-start; gap: var(--s-3); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  section > :global(*) { align-self: stretch; }
  section > :global(.btn) { align-self: flex-start; }
  h2 { font: var(--t-h3); margin: 0; }
  .confirm { display: flex; flex-direction: column; gap: var(--s-3); }
  dl { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--s-3); margin: 0; }
  dt { color: var(--text-3); font: var(--t-small); }
  dd { margin: var(--s-1) 0 0; overflow-wrap: anywhere; }
  .mono { font-family: var(--font-mono); }
  .muted { margin: 0; color: var(--text-3); }
  .total { margin: 0; }
  .total .mono { font: var(--t-h3); font-family: var(--font-mono); margin-right: var(--s-1); }
  .bars { width: 100%; height: 60px; }
  .bars rect { fill: var(--primary); }
  .flows, .roles { list-style: none; margin: 0; padding: 0; }
  .flows li, .roles li { display: flex; justify-content: space-between; padding: var(--s-1) 0; border-bottom: 1px solid var(--border); }
  .flows a { color: var(--text); }
  .caps { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--s-3); margin: 0; }
  .cap-form { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--s-3); align-items: end; }
  .cap-form :global(.btn) { grid-column: 1 / -1; justify-self: start; }
  h3 { font: var(--t-body); margin: 0; }
  .create { display: flex; align-items: flex-end; gap: var(--s-2); }
  .create :global(.field) { flex: 1; }
  .row { display: flex; flex-wrap: wrap; gap: var(--s-2); }
  .keys { list-style: none; margin: 0; padding: 0; }
  .keys li { display: flex; align-items: center; gap: var(--s-3); padding: var(--s-2) 0; border-bottom: 1px solid var(--border); }
  .name { flex: 1; font-weight: 600; }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); }
    dl { grid-template-columns: minmax(0, 1fr); }
    .cap-form { grid-template-columns: minmax(0, 1fr); }
    .create { flex-direction: column; align-items: stretch; }
  }
</style>
