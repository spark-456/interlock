<script lang="ts">
  // App shell (W01): Sidebar (240 / 64 px on canvas screens), page content, mobile TabBar.
  import '../app.css';
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, ApiRequestError, type Me } from '$lib/api';
  import Banner from '$lib/design/Banner.svelte';
  import Sidebar from '$lib/design/Sidebar.svelte';
  import TabBar from '$lib/design/TabBar.svelte';
  import Toast from '$lib/design/Toast.svelte';
  import { isCanvasRoute } from '$lib/design/nav';
  import Onboarding from '$lib/onboarding/Onboarding.svelte';
  import { onboarding } from '$lib/onboarding/state.svelte';

  let { children }: { children: Snippet } = $props();
  let me = $state<Me | null>(null);

  $effect(() => {
    void page.url.pathname;
    api.me().then(
      (m) => (me = m),
      async (e) => {
        me = null;
        if (!(e instanceof ApiRequestError) || e.status !== 401) return;
        // An expired session would leave every screen empty: drop its cookie and reload, which
        // can't repeat because the cookie is gone.
        if (e.code === 'session_expired') {
          await api.signOut().catch(() => {});
          location.reload();
          return;
        }
        if (page.route.id !== '/') await goto('/');
      }
    );
  });

  // The offline banner follows the browser's connection state.
  let online = $state(typeof navigator === 'undefined' || navigator.onLine);

  const user = $derived(me?.user?.name ?? 'Guest');
</script>

{#if page.route.id === '/'}
  <!-- The landing page brings its own header. -->
  {@render children()}
{:else}
<div class="app">
  <Sidebar collapsed={isCanvasRoute(page.route.id)} path={page.url.pathname} {user} plan="Free plan" onhelp={() => (onboarding.helpOpen = true)} />
  <div class="content">
    {#if !online}<Banner kind="offline">You're offline. Saving, builds and runs work again once you reconnect.</Banner>{/if}
    <main>{@render children()}</main>
    <TabBar path={page.url.pathname} />
  </div>
</div>
<Onboarding {me} />
{/if}
<Toast />
<svelte:window ononline={() => (online = true)} onoffline={() => (online = false)} />

<style>
  .app { display: flex; min-height: 100vh; }
  .content { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; }
  main { flex: 1 1 auto; min-height: 0; display: flex; flex-direction: column; }
</style>
