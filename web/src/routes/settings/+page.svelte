<script lang="ts">
  // Settings: the workspace you're in, API keys (create once, revoke), and how spending is capped.
  import { api, type Me } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import Input from '$lib/design/Input.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import { ago } from '../flows/flows';

  type Key = Awaited<ReturnType<typeof api.keys>>[number];
  let me = $state.raw<Me | null>(null);
  let keys = $state.raw<Key[] | null>(null);
  let keysError = $state<string | null>(null);
  let name = $state('');
  let created = $state<string | null>(null);
  let busy = $state<string | null>(null);
  const now = new Date();

  api.me().then((m) => (me = m), () => {});
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
</script>

<svelte:head><title>Settings · Interlock</title></svelte:head>

<div class="page">
  <h1>Settings</h1>

  <section aria-labelledby="ws">
    <h2 id="ws">Workspace</h2>
    {#if me}
      <dl>
        <div><dt>Signed in as</dt><dd>{me.user?.name ?? (me.demo ? 'Demo visitor' : 'Guest')}</dd></div>
        <div><dt>Workspace</dt><dd class="mono">{me.workspaceId}</dd></div>
        <div><dt>Plan</dt><dd>{me.demo ? 'Demo workspace (resets daily)' : 'Free plan'}</dd></div>
      </dl>
      {#if !me.user}<Button variant="primary" href="/api/auth/github/start">Sign in with GitHub</Button>{/if}
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

  <section aria-labelledby="caps">
    <h2 id="caps">Usage and caps</h2>
    <p class="muted">
      Every run is capped: per run, per flow per day and per workspace per day. A flow can set its own lower per-run cap in its settings.
      When a cap is reached the run stops and says so.{#if me?.demo} Demo workspaces use small caps, the smaller models and recorded media.{/if}
    </p>
  </section>
</div>

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
  dl { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--s-3); margin: 0; }
  dt { color: var(--text-3); font: var(--t-small); }
  dd { margin: var(--s-1) 0 0; overflow-wrap: anywhere; }
  .mono { font-family: var(--font-mono); }
  .muted { margin: 0; color: var(--text-3); }
  .create { display: flex; align-items: flex-end; gap: var(--s-2); }
  .create :global(.field) { flex: 1; }
  .keys { list-style: none; margin: 0; padding: 0; }
  .keys li { display: flex; align-items: center; gap: var(--s-3); padding: var(--s-2) 0; border-bottom: 1px solid var(--border); }
  .name { flex: 1; font-weight: 600; }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); }
    dl { grid-template-columns: minmax(0, 1fr); }
    .create { flex-direction: column; align-items: stretch; }
  }
</style>
