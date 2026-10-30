<script lang="ts">
  // App shell (W01): Sidebar (240 / 64 px on canvas screens), page content, mobile TabBar.
  import '../app.css';
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { api, ApiRequestError, type Me } from '$lib/api';
  import Banner from '$lib/design/Banner.svelte';
  import Button from '$lib/design/Button.svelte';
  import Sidebar from '$lib/design/Sidebar.svelte';
  import TabBar from '$lib/design/TabBar.svelte';
  import Toast from '$lib/design/Toast.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import { isCanvasRoute } from '$lib/design/nav';
  import Onboarding from '$lib/onboarding/Onboarding.svelte';
  import { onboarding } from '$lib/onboarding/state.svelte';

  let { children }: { children: Snippet } = $props();
  let me = $state<Me | null>(null);
  let leaving = $state(false);

  $effect(() => {
    void page.url.pathname;
    api.me().then(
      (m) => (me = m),
      async (e) => {
        me = null;
        if (!(e instanceof ApiRequestError) || e.status !== 401) return;
        // An expired demo session (demos reset daily) would leave every screen empty: drop its cookie,
        // then continue in the local workspace, or go home to start a new demo.
        // The reload refetches every screen's data without the cookie; it can't repeat, the cookie is gone.
        if (e.code === 'session_expired') {
          await api.leaveDemo().catch(() => {});
          location.reload();
          return;
        }
        if (page.route.id !== '/' && page.route.id !== '/demo') await goto('/');
      }
    );
  });

  async function leaveDemo() {
    leaving = true;
    try {
      await api.leaveDemo();
      me = await api.me();
      await goto('/flows');
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not leave the demo.', 'error');
    } finally {
      leaving = false;
    }
  }

  // The offline banner follows the browser's connection state.
  let online = $state(typeof navigator === 'undefined' || navigator.onLine);

  const user = $derived(me?.user?.name ?? (me?.demo ? 'Demo visitor' : 'Guest'));
  const plan = $derived(me?.demo ? 'Demo workspace' : 'Free plan');
</script>

{#if page.route.id === '/' || page.route.id === '/demo'}
  <!-- The landing and demo pages bring their own header. -->
  {@render children()}
{:else}
<div class="app">
  <Sidebar collapsed={isCanvasRoute(page.route.id)} path={page.url.pathname} {user} {plan} onhelp={() => (onboarding.helpOpen = true)} />
  <div class="content">
    {#if !online}<Banner kind="offline">You're offline. Saving, builds and runs work again once you reconnect.</Banner>{/if}
    {#if me?.demo}
      <Banner kind="demo">
        You're in the demo. Builds use recorded media and small limits.
        {#snippet action()}<Button size="s" loading={leaving} onclick={leaveDemo}>Leave demo</Button>{/snippet}
      </Banner>
    {/if}
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
