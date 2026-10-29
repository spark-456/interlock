<script lang="ts">
  // Templates gallery: ready-made flows; "Use template" copies one into the workspace and opens it.
  import { goto } from '$app/navigation';
  import { api, appNames as loadAppNames } from '$lib/api';
  import type { TemplateSummary } from '$lib/api/types';
  import AppTile from '$lib/design/AppTile.svelte';
  import Button from '$lib/design/Button.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import CardSkeleton from '$lib/design/CardSkeleton.svelte';

  let templates = $state.raw<TemplateSummary[] | null>(null);
  let failed = $state(false);
  let appNames = $state<Record<string, string>>({});
  let using = $state<string | null>(null);

  api.templates().then((t) => (templates = t), () => (failed = true));
  loadAppNames().then((names) => (appNames = names), () => {});

  async function use(t: TemplateSummary) {
    using = t.id;
    try {
      const flow = await api.useTemplate(t.id);
      await goto(`/flows/${flow.id}`);
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not use the template.', 'error');
    } finally {
      using = null;
    }
  }
</script>

<svelte:head><title>Templates · Interlock</title></svelte:head>

<div class="page">
  <header>
    <h1>Templates</h1>
    <p>Start from a flow that already works. Each one opens in the editor, ready to connect your apps.</p>
  </header>

  {#if failed}
    <p role="alert" class="error">Could not load templates. Refresh to try again.</p>
  {:else if templates}
    <div class="grid">
      {#each templates as t (t.id)}
        <article class="card">
          <div class="apps" role="group" aria-label="Apps used">
            {#each t.apps.slice(0, 5) as a (a)}<span title={appNames[a] ?? a}><AppTile name={appNames[a] ?? a} size={28} /></span>{/each}
            {#if t.apps.length > 5}<span class="more">+{t.apps.length - 5}</span>{/if}
          </div>
          <h2>{t.name}</h2>
          <p class="kind">{t.kind === 'app' ? 'AI app' : 'Automation'}</p>
          <p>{t.description}</p>
          <footer>
            {#if t.hasRecordedRun}<Chip kind="source" icon="play">Recorded run</Chip>{:else}<span></span>{/if}
            <Button variant="primary" loading={using === t.id} disabled={!!using} onclick={() => use(t)}>Use template</Button>
          </footer>
        </article>
      {:else}
        <p class="none">No templates yet.</p>
      {/each}
    </div>
  {:else}
    <CardSkeleton count={6} label="Loading templates" />
  {/if}
</div>

<style>
  .page { max-width: 1120px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-5); }
  h1 { font: var(--t-h1); margin: 0; }
  header p { margin: var(--s-2) 0 0; color: var(--text-2); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: var(--s-5); }
  .card {
    display: flex; flex-direction: column; gap: var(--s-2); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  .apps { display: flex; gap: var(--s-1); align-items: center; margin-bottom: var(--s-2); }
  .more, .kind, .none { color: var(--text-3); }
  h2 { font: var(--t-h3); margin: 0; }
  .kind { margin: 0; font: var(--t-small); }
  .card > p:not(.kind) { margin: 0; color: var(--text-2); flex: 1; }
  footer { display: flex; justify-content: space-between; align-items: center; margin-top: var(--s-3); }
  .error { color: var(--error-text); }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); }
  }
</style>
