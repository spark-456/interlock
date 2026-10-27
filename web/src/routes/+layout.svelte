<script lang="ts">
  // App shell (W01): Sidebar (240 / 64 px on canvas screens), page content, mobile TabBar.
  import '../app.css';
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import { api, type Me } from '$lib/api';
  import Banner from '$lib/design/Banner.svelte';
  import Sidebar from '$lib/design/Sidebar.svelte';
  import TabBar from '$lib/design/TabBar.svelte';
  import Toast from '$lib/design/Toast.svelte';
  import { isCanvasRoute } from '$lib/design/nav';

  let { children }: { children: Snippet } = $props();
  let me = $state<Me | null>(null);

  $effect(() => {
    api.me().then((m) => (me = m)).catch(() => (me = null));
  });

  const user = $derived(me?.user?.name ?? (me?.demo ? 'Demo visitor' : 'Guest'));
  const plan = $derived(me?.demo ? 'Demo workspace' : 'Free plan');
</script>

<div class="app">
  <Sidebar collapsed={isCanvasRoute(page.route.id)} path={page.url.pathname} {user} {plan} />
  <div class="content">
    {#if me?.demo}<Banner kind="demo">You're in the demo. Builds use recorded media and small limits.</Banner>{/if}
    <main>{@render children()}</main>
    <TabBar path={page.url.pathname} />
  </div>
</div>
<Toast />

<style>
  .app { display: flex; min-height: 100vh; }
  .content { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; }
  main { flex: 1 1 auto; min-height: 0; display: flex; flex-direction: column; }
</style>
