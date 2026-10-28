<script lang="ts">
  // Connections: every app the catalog can use, its state, and a detail drawer to connect it.
  import { api } from '$lib/api';
  import type { FlowSummary } from '$lib/api/types';
  import AppTile from '$lib/design/AppTile.svelte';
  import Button from '$lib/design/Button.svelte';
  import Modal from '$lib/design/Modal.svelte';
  import StatusPill, { type PillTone } from '$lib/design/StatusPill.svelte';
  import Input from '$lib/design/Input.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import { appRows, connectHref, keyFields, takesKey, type AppRow } from './connections';

  let rows = $state.raw<AppRow[] | null>(null);
  let failed = $state(false);
  let flows = $state.raw<FlowSummary[]>([]);
  let open = $state<AppRow | null>(null);
  let drawer = $state(false);

  const load = () =>
    Promise.all([api.nodes(), api.connections()]).then(([c, conns]) => (rows = appRows(c.nodes, conns)), () => (failed = true));
  load();

  // The key form for apps that use a key, token, password or webhook instead of a sign-in.
  let values = $state<Record<string, string>>({});
  let label = $state('');
  let saving = $state(false);

  async function save(row: AppRow) {
    saving = true;
    try {
      await api.createConnection({ app: row.app, label: label.trim() || undefined, secret: { ...values } });
      toast(`${row.name} is connected.`, 'success');
      drawer = false;
      await load();
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not save the connection.', 'error');
    } finally {
      saving = false;
    }
  }
  api.flows().then((f) => (flows = f), () => {});

  const STATUS: Record<AppRow['status'], [PillTone, string]> = {
    connected: ['ready', 'Connected'],
    expired: ['warning', 'Expired'],
    notConnected: ['waiting', 'Not connected']
  };
  // Flow apps are node apps (gmail, sheets); Google covers those three.
  const usedBy = (row: AppRow) =>
    flows.filter((f) => f.apps.some((a) => a === row.app || (row.app === 'google' && ['gmail', 'sheets', 'youtube'].includes(a))));

  function show(row: AppRow) {
    open = row;
    values = Object.fromEntries(keyFields(row).map((f) => [f.key, '']));
    label = '';
    drawer = true;
  }
</script>

<svelte:head><title>Connections · Interlock</title></svelte:head>

<div class="page">
  <header>
    <h1>Connections</h1>
    <p>Connect the apps your flows post to and read from. Tokens stay encrypted on the server and never reach the browser.</p>
  </header>

  {#if failed}
    <p role="alert" class="error">Could not load connections. Refresh to try again.</p>
  {:else if rows}
    <ul class="grid" aria-label="Apps">
      {#each rows as row (row.app)}
        <li>
          <button type="button" class="tile" onclick={() => show(row)} aria-label="{row.name}, {STATUS[row.status][1]}">
            <AppTile name={row.name} size={36} />
            <span class="text">
              <span class="name">{row.name}</span>
              <span class="account">{row.account ?? row.label}</span>
            </span>
            <StatusPill tone={STATUS[row.status][0]} label={STATUS[row.status][1]} />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if open}
  <Modal bind:open={drawer} title={open.name}>
    <dl>
      <div><dt>Status</dt><dd><StatusPill tone={STATUS[open.status][0]} label={STATUS[open.status][1]} /></dd></div>
      <div><dt>Connects</dt><dd>{open.label}</dd></div>
      {#if open.account}<div><dt>Account</dt><dd>{open.account}</dd></div>{/if}
      <div>
        <dt>Used by</dt>
        <dd>
          {#each usedBy(open) as f (f.id)}<a href="/flows/{f.id}">{f.name}</a>{:else}No flows yet{/each}
        </dd>
      </div>
    </dl>
    {#if !connectHref(open) && !takesKey(open)}
      <p class="hint">Connecting {open.name} isn't available yet. Its steps run as dry runs: they show what would be posted.</p>
    {:else if !connectHref(open)}
      {@const row = open}
      <form id="key-form" class="keys" onsubmit={(e) => (e.preventDefault(), save(row))}>
        <p class="hint">{row.name} uses a {row.label.toLowerCase()} rather than a sign-in. It's encrypted on the server and never shown again.</p>
        {#each keyFields(row) as f (f.key)}
          <Input label={f.label} placeholder={f.placeholder} type={f.secret ? 'password' : 'text'} autocomplete="off" required bind:value={values[f.key]} />
        {/each}
        <Input label="Name (optional)" placeholder="{row.name} for support" bind:value={label} />
      </form>
    {/if}
    {#snippet footer()}
      {#if open && connectHref(open)}
        <Button variant="primary" href={connectHref(open) ?? undefined}>{open.status === 'notConnected' ? 'Connect' : 'Reconnect'}</Button>
      {:else if open && takesKey(open)}
        <Button variant="primary" type="submit" form="key-form" loading={saving}>Save</Button>
      {/if}
    {/snippet}
  </Modal>
{/if}

<style>
  .page { max-width: 1120px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-5); }
  h1 { font: var(--t-h1); margin: 0; }
  header p { margin: var(--s-2) 0 0; color: var(--text-2); max-width: 640px; }
  .grid { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--s-4); }
  .tile {
    width: 100%; display: flex; align-items: center; gap: var(--s-3); padding: var(--s-4); text-align: left; font: inherit; color: inherit; cursor: pointer;
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  .tile:hover { box-shadow: var(--shadow-pop-hover); }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .name { font-weight: 600; }
  .account { color: var(--text-3); font: var(--t-small); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  dl { display: flex; flex-direction: column; gap: var(--s-3); margin: 0; }
  dt { color: var(--text-3); font: var(--t-small); }
  dd { margin: var(--s-1) 0 0; display: flex; flex-wrap: wrap; gap: var(--s-2); }
  dd a { color: var(--primary); }
  .hint { color: var(--text-2); margin: 0; }
  .keys { display: flex; flex-direction: column; gap: var(--s-3); margin-top: var(--s-4); }
  .error { color: var(--error-text); }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); }
  }
</style>
