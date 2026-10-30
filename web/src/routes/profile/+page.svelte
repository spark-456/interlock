<script lang="ts">
  // Profile (Q41): who is signed in and how, signing out, and deleting the account. Without a
  // sign-in this is the computer's guest workspace, with the way to sign in.
  import { api, type Me } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import Input from '$lib/design/Input.svelte';
  import Modal from '$lib/design/Modal.svelte';
  import Skeleton from '$lib/design/Skeleton.svelte';
  import { initials } from '$lib/design/format';
  import { toast } from '$lib/design/toasts.svelte';

  let me = $state.raw<Me | null>(null);
  let busy = $state<'signout' | 'delete' | null>(null);
  let confirmingDelete = $state(false);
  let deleteText = $state('');
  let broken = $state(false);

  api.me().then((m) => (me = m), () => {});

  const provider = $derived(
    me?.user?.id.startsWith('google:') ? 'Google' : me?.user?.id.startsWith('github:') ? 'GitHub' : null
  );

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

<svelte:head><title>Profile · Interlock</title></svelte:head>

<div class="page">
  <h1>Profile</h1>

  <section aria-labelledby="me">
    {#if !me}
      <Skeleton lines={3} />
    {:else if me.user}
      <div class="person">
        {#if me.user.avatarUrl && !broken}
          <img class="avatar" src={me.user.avatarUrl} alt="" referrerpolicy="no-referrer" onerror={() => (broken = true)} />
        {:else}
          <span class="avatar" aria-hidden="true">{initials(me.user.name)}</span>
        {/if}
        <div>
          <h2 id="me">{me.user.name}</h2>
          <p class="muted">{me.user.email ?? 'Email not shared'}</p>
        </div>
      </div>
      <dl>
        {#if provider}<div><dt>Signed in with</dt><dd>{provider}</dd></div>{/if}
        <div><dt>Workspace</dt><dd class="mono">{me.workspaceId}</dd></div>
        <div><dt>Plan</dt><dd>Free plan</dd></div>
      </dl>
      <Button icon="sign-out" loading={busy === 'signout'} onclick={signOut}>Sign out</Button>
    {:else}
      <h2 id="me">Guest</h2>
      <p class="muted">You're using this computer's guest workspace, which only this computer can open. Sign in to get a workspace of your own that you can use anywhere.</p>
      <Button variant="primary" icon="sign-in" href="/signin">Sign in</Button>
    {/if}
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
    display: flex; flex-direction: column; align-items: flex-start; gap: var(--s-4); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  h2 { font: var(--t-h3); margin: 0; }
  .person { display: flex; align-items: center; gap: var(--s-4); }
  .avatar {
    width: 64px; height: 64px; flex: none; border-radius: 50%; background: var(--primary-soft); color: var(--primary);
    font-weight: 700; font-size: 22px; display: flex; align-items: center; justify-content: center; object-fit: cover;
  }
  dl { align-self: stretch; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--s-3); margin: 0; }
  dt { color: var(--text-3); font: var(--t-small); }
  dd { margin: var(--s-1) 0 0; overflow-wrap: anywhere; }
  .mono { font-family: var(--font-mono); }
  .muted { margin: 0; color: var(--text-3); }
  .confirm { display: flex; flex-direction: column; gap: var(--s-3); }
  @media (max-width: 767px) {
    .page { padding: var(--s-5) var(--s-4) var(--s-10); }
    dl { grid-template-columns: 1fr; }
  }
</style>
