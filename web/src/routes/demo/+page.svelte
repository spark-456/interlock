<script lang="ts">
  // Demo picker (design S8): no account. Opening anything first starts a demo workspace
  // (POST /api/demo/session), which comes seeded with the templates.
  import { goto } from '$app/navigation';
  import { api, appNames as loadAppNames } from '$lib/api';
  import type { TemplateSummary } from '$lib/api/types';
  import AppTile from '$lib/design/AppTile.svelte';
  import Banner from '$lib/design/Banner.svelte';
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import Logo from '$lib/design/Logo.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import CardSkeleton from '$lib/design/CardSkeleton.svelte';

  let templates = $state.raw<TemplateSummary[] | null>(null);
  let appNames = $state<Record<string, string>>({});
  let opening = $state<string | null>(null);
  let session: Promise<unknown> | null = null;

  api.templates().then((t) => (templates = t), () => (templates = []));
  loadAppNames().then((names) => (appNames = names), () => {});

  /** A workspace to open things in: the visitor's own, or a new demo one. */
  function ensureSession() {
    session ??= api.me().then((me) => me, () => null).then((me) => (me?.demo || me?.user ? me : api.demoSession()));
    return session;
  }

  async function open(id: string, go: () => Promise<string>) {
    opening = id;
    try {
      await ensureSession();
      await goto(await go());
    } catch (e) {
      session = null;
      toast(e instanceof Error ? e.message : 'Could not open the demo.', 'error');
    } finally {
      opening = null;
    }
  }

  // The demo workspace is seeded with every template; open that copy, or copy it now.
  const openTemplate = (t: TemplateSummary) =>
    open(t.id, async () => {
      const seeded = (await api.flows()).find((f) => f.name === t.name);
      const id = seeded?.id ?? (await api.useTemplate(t.id)).id;
      return t.kind === 'app' ? `/flows/${id}/chat` : `/flows/${id}`;
    });
</script>

<svelte:head><title>Try the demo · Interlock</title></svelte:head>

<header class="nav">
  <a href="/" class="brand" aria-label="Interlock home"><Logo variant="lockup" size={28} /></a>
  <span class="tag">Demo</span>
  <a class="how" href="/#how">How it works</a>
  <Button size="s" href="/api/auth/github/start">Sign in</Button>
</header>
<Banner kind="demo">Demo · sample data · resets daily</Banner>

<main class="page">
  <h1>Try Interlock without an account.</h1>
  <p class="lede">Open a ready-made flow instantly, or build your own from your goal, documents and one API.</p>

  {#if !templates}<CardSkeleton count={3} label="Loading demos" />{/if}
  <div class="grid">
    {#each templates ?? [] as t (t.id)}
      <article class="card">
        <div class="apps" role="group" aria-label="Apps used">
          {#each t.apps.slice(0, 6) as a (a)}<span title={appNames[a] ?? a}><AppTile name={appNames[a] ?? a} size={28} /></span>{/each}
        </div>
        <div class="title">
          <h2>{t.name}</h2>
          <span class="kind">{t.kind === 'app' ? 'AI app' : 'Automation'}</span>
          {#if t.hasRecordedRun}<span class="instant"><Icon name="bolt" size={14} /> Opens instantly</span>{/if}
        </div>
        <p>{t.description}</p>
        <footer>
          <Button variant="primary" loading={opening === t.id} disabled={!!opening} onclick={() => openTemplate(t)}>Open <Icon name="arrow-right" size={16} /></Button>
        </footer>
      </article>
    {/each}
    <article class="card own">
      <span class="pill"><span class="dot"></span> Live build · capped</span>
      <div class="plus" aria-hidden="true"><Icon name="plus" size={22} /></div>
      <h2>Build your own</h2>
      <p>Bring a goal, a few documents and one API. Watch Interlock build and test it in about two minutes.</p>
      <footer>
        <span class="limits">Up to 20 pages and 1 API</span>
        <Button variant="primary" loading={opening === 'own'} disabled={!!opening} onclick={() => open('own', async () => '/new')}>Start <Icon name="arrow-right" size={16} /></Button>
      </footer>
    </article>
  </div>
  <p class="fine">All demo companies and data are fictional. Nothing you type in the demo is kept after the daily reset.</p>
</main>

<style>
  .nav { display: flex; align-items: center; gap: var(--s-3); padding: var(--s-3) var(--s-6); background: var(--surface); border-bottom: 1px solid var(--border); }
  .brand { display: inline-flex; }
  .tag { padding: 0 var(--s-2); border-radius: var(--r-pill); background: var(--surface-2); color: var(--text-2); font: var(--t-small); }
  .how { margin-left: auto; color: var(--text-2); text-decoration: none; }
  .page { max-width: 1072px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-4); }
  h1 { font: var(--t-display); margin: 0; }
  .lede { margin: 0 0 var(--s-4); color: var(--text-2); }
  .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--s-5); }
  .card {
    display: flex; flex-direction: column; gap: var(--s-3); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  .card.own { border-color: var(--primary-line); }
  .apps { display: flex; gap: var(--s-1); }
  .title { display: flex; align-items: center; gap: var(--s-2); flex-wrap: wrap; }
  h2 { font: var(--t-h3); margin: 0; }
  .kind { padding: 0 var(--s-2); border-radius: var(--r-pill); background: var(--cat-kb-tile); color: var(--cat-kb-icon); font: var(--t-small); }
  .instant { margin-left: auto; display: inline-flex; align-items: center; gap: var(--s-1); color: var(--text-3); font: var(--t-small); }
  .card p { margin: 0; color: var(--text-2); flex: 1; }
  footer { display: flex; justify-content: flex-end; align-items: center; gap: var(--s-3); padding-top: var(--s-3); border-top: 1px solid var(--border); }
  .pill { align-self: flex-start; display: inline-flex; align-items: center; gap: var(--s-1); padding: 0 var(--s-2); border: 1px solid var(--primary-line); border-radius: var(--r-pill); color: var(--primary); font: var(--t-small); }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--primary); }
  .plus { align-self: center; display: grid; place-items: center; width: 40px; height: 40px; border-radius: var(--r-card); background: var(--primary); color: var(--surface); box-shadow: var(--shadow-button-primary); }
  .limits { margin-right: auto; color: var(--text-3); font: var(--t-small); }
  .fine { text-align: center; color: var(--text-3); font: var(--t-small); }
  @media (max-width: 767px) {
    .nav { padding: var(--s-3) var(--s-4); }
    .how { display: none; }
    .page { padding: var(--s-5) var(--s-4); }
    .grid { grid-template-columns: minmax(0, 1fr); }
  }
</style>
