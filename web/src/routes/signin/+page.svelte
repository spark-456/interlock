<script lang="ts">
  // Sign in (WS-1): one button per provider this server is set up for. The first sign-in creates
  // the person's own workspace. A failed sign-in comes back here with `?error=`.
  import { page } from '$app/state';
  import { api, type AuthProvider } from '$lib/api';
  import AppTile from '$lib/design/AppTile.svelte';
  import Logo from '$lib/design/Logo.svelte';
  import Skeleton from '$lib/design/Skeleton.svelte';

  let providers = $state<AuthProvider[] | null>(null);
  let failed = $state(false);
  const error = $derived(page.url.searchParams.get('error'));

  api.authProviders().then((p) => (providers = p), () => (failed = true));
</script>

<svelte:head><title>Sign in · Interlock</title></svelte:head>

<main class="signin">
  <a href="/" class="home" aria-label="Interlock home"><Logo variant="lockup" size={32} /></a>
  <section class="card" aria-labelledby="signin-title">
    <h1 id="signin-title">Sign in to Interlock</h1>
    <p class="lede">Your flows, connections and keys live in your own workspace. Signing in the first time creates it.</p>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    {#if providers?.length}
      <ul class="providers">
        {#each providers as p (p.id)}
          <li><a class="provider" href="/api/auth/{p.id}/start"><AppTile name={p.label} size={28} /> Continue with {p.label}</a></li>
        {/each}
      </ul>
    {:else if providers || failed}
      <p class="muted">Sign-in isn't set up on this server yet. Its owner needs to add a Google or GitHub OAuth client.</p>
    {:else}
      <Skeleton lines={2} />
    {/if}
    <p class="fine">Signing in shares only your name, email and picture. Gmail, Drive and other apps connect separately, only when you choose to.</p>
  </section>
</main>

<style>
  .signin {
    min-height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: var(--s-6); padding: var(--s-6) var(--s-4); background: var(--bg);
  }
  .card {
    width: min(420px, 100%); display: flex; flex-direction: column; gap: var(--s-4); padding: var(--s-6);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  h1 { font: var(--t-h2); margin: 0; }
  .lede, .fine, .muted { margin: 0; color: var(--text-2); }
  .fine { font: var(--t-small); color: var(--text-3); }
  .error { margin: 0; padding: var(--s-2) var(--s-3); border-radius: var(--r-control); background: var(--error-soft); color: var(--error-text); }
  .providers { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--s-3); }
  .provider {
    display: flex; align-items: center; justify-content: center; gap: var(--s-3); min-height: 48px; padding: 0 var(--s-4);
    border: 1px solid var(--border-strong); border-radius: var(--r-control); background: var(--surface); color: var(--text);
    font-weight: 600; text-decoration: none;
  }
  .provider:hover { background: var(--surface-2); }
  .provider:focus-visible { outline: none; box-shadow: var(--focus-ring); }
</style>
