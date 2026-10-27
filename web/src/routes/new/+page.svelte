<script lang="ts">
  // S2 Create (AR-1): the goal (typed, picked from an example or spoken), what it should know
  // (files, links, voice memos as Knowledge or Instructions), and the apps and APIs it may use.
  import { goto } from '$app/navigation';
  import { api, appNames as loadAppNames } from '$lib/api';
  import AppTile from '$lib/design/AppTile.svelte';
  import Button from '$lib/design/Button.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import Input from '$lib/design/Input.svelte';
  import TextArea from '$lib/design/TextArea.svelte';
  import TopBar from '$lib/design/TopBar.svelte';
  import { toast } from '$lib/design/toasts.svelte';
  import AttachmentRow from './AttachmentRow.svelte';
  import VoiceButton from './VoiceButton.svelte';
  import { EXAMPLES, MAX_BYTES, buildRequest, detectApps, fileDraft, linkDraft, uploaded, type Draft } from './create';

  let goal = $state('');
  let drafts = $state<Draft[]>([]);
  let link = $state('');
  let linkError = $state('');
  let spec = $state('');
  let specError = $state('');
  let skipped = $state<string[]>([]);
  let dragging = $state(false);
  let starting = $state(false);
  let appNames = $state<Record<string, string>>({});
  let filePicker = $state<HTMLInputElement>();
  let specPicker = $state<HTMLInputElement>();

  loadAppNames().then((names) => (appNames = names), () => {});

  const knowledge = $derived(drafts.filter((d) => d.kind !== 'api'));
  const apis = $derived(drafts.filter((d) => d.kind === 'api'));
  const detected = $derived(detectApps(goal, Object.keys(appNames)));
  const uploading = $derived(drafts.some((d) => d.uploading));
  const canBuild = $derived(goal.trim().length > 0 && !uploading && !starting);

  const update = (key: string, patch: Partial<Draft>) => (drafts = drafts.map((d) => (d.key === key ? { ...d, ...patch } : d)));
  const remove = (key: string) => (drafts = drafts.filter((d) => d.key !== key));

  async function addFiles(files: File[], asSpec = false) {
    const rows = files.map((f) => {
      const d = fileDraft(f);
      return asSpec ? { ...d, kind: 'api' as const, detail: 'OpenAPI spec · endpoints are chosen for your goal' } : d;
    });
    drafts = [...drafts, ...rows];
    await Promise.all(
      rows.map(async (row, i) => {
        if (row.error) return;
        try {
          const [ref] = await api.uploadFiles([files[i]!]);
          drafts = drafts.map((d) => (d.key === row.key && ref ? uploaded(d, ref) : d));
        } catch (e) {
          update(row.key, { uploading: false, error: (e as Error).message });
        }
      })
    );
  }

  function addLink(kind: 'link' | 'api') {
    const d = linkDraft(kind === 'link' ? link : spec, kind);
    if (kind === 'link') linkError = typeof d === 'string' ? d : '';
    else specError = typeof d === 'string' ? d : '';
    if (typeof d === 'string') return;
    drafts = [...drafts, d];
    if (kind === 'link') link = '';
    else spec = '';
  }

  function drop(e: DragEvent) {
    e.preventDefault();
    dragging = false;
    const files = [...(e.dataTransfer?.files ?? [])];
    const uri = e.dataTransfer?.getData('text/uri-list');
    if (files.length) void addFiles(files);
    else if (uri) {
      link = uri;
      addLink('link');
    }
  }

  function picked(input: HTMLInputElement | undefined, asSpec = false) {
    const files = [...(input?.files ?? [])];
    if (input) input.value = '';
    if (files.length) void addFiles(files, asSpec);
  }

  async function build() {
    if (!canBuild) return;
    starting = true;
    try {
      const hints = detected.filter((a) => !skipped.includes(a));
      const { buildId } = await api.startBuild(buildRequest(goal, drafts, hints));
      await goto(`/new/${buildId}/building`);
    } catch (e) {
      toast((e as Error).message, 'error');
      starting = false;
    }
  }

  function keys(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      void build();
    }
  }
</script>

<svelte:head><title>New flow · Interlock</title></svelte:head>
<svelte:window onkeydown={keys} />

<TopBar title="New flow" crumb={{ href: '/flows', label: 'Flows' }} back="/flows" />

<div class="page">
  <header>
    <h1>New flow</h1>
    <p>Tell it what to do, what to know and which apps it may use. Interlock builds and tests the rest.</p>
  </header>

  <section class="card" aria-labelledby="s1">
    <h2 id="s1"><span class="num">1</span>What should it do?</h2>
    <div class="goal">
      <TextArea
        label="Goal"
        rows={4}
        bind:value={goal}
        placeholder="e.g. Every morning, summarise new support emails and post the summary in Slack."
      />
      <div class="voice">
        <VoiceButton ondictate={(t) => (goal = goal.trim() ? `${goal.trim()} ${t}` : t)} onrecorded={(f) => addFiles([f])} />
      </div>
    </div>
    <div class="examples">
      <span class="muted">Or start from</span>
      {#each EXAMPLES as ex (ex.label)}
        <Chip icon={ex.icon} onclick={() => (goal = ex.goal)}>{ex.label}</Chip>
      {/each}
    </div>
  </section>

  <section class="card" aria-labelledby="s2">
    <h2 id="s2"><span class="num">2</span>What should it know?<span class="aside">Optional</span></h2>
    <div
      class="drop"
      class:dragging
      role="region"
      aria-label="Drop files here"
      ondragover={(e) => {
        e.preventDefault();
        dragging = true;
      }}
      ondragleave={() => (dragging = false)}
      ondrop={drop}
    >
      <span class="drop-icon"><Icon name="upload" size={20} /></span>
      <div class="drop-text">
        <strong>Drop files or paste a link</strong>
        <span class="muted">PDF, DOCX, Markdown, TXT, CSV or a voice memo, up to {MAX_BYTES / 1024 / 1024} MB. Websites are crawled for you.</span>
      </div>
      <Button size="s" onclick={() => filePicker?.click()}>Browse files</Button>
      <input
        bind:this={filePicker}
        type="file"
        multiple
        hidden
        accept=".pdf,.docx,.md,.txt,.csv,audio/*"
        data-testid="file-input"
        onchange={() => picked(filePicker)}
      />
    </div>
    <form
      class="add"
      onsubmit={(e) => {
        e.preventDefault();
        addLink('link');
      }}
    >
      <Input label="Link" placeholder="https://help.example.com" bind:value={link} error={linkError || undefined} />
      <Button type="submit" disabled={!link.trim()}>Add link</Button>
    </form>
    {#if knowledge.length}
      <p class="note"><Icon name="info" size={14} /> Knowledge is searched when needed. Instructions are always followed.</p>
      <ul class="list" aria-label="Attachments">
        {#each knowledge as d (d.key)}
          <AttachmentRow draft={d} onrole={(role) => update(d.key, { role })} onremove={() => remove(d.key)} />
        {/each}
      </ul>
    {/if}
  </section>

  <section class="card" aria-labelledby="s3">
    <h2 id="s3"><span class="num">3</span>Which apps and APIs can it use?<span class="aside">Optional</span></h2>
    {#if detected.length}
      <div class="apps" role="group" aria-label="Apps found in your goal">
        {#each detected as app (app)}
          {@const on = !skipped.includes(app)}
          <button
            type="button"
            class="app"
            class:off={!on}
            aria-pressed={on}
            title={on ? 'Click to leave it out' : 'Click to include it'}
            onclick={() => (skipped = on ? [...skipped, app] : skipped.filter((a) => a !== app))}
          >
            <span aria-hidden="true"><AppTile name={appNames[app] ?? app} size={36} /></span><span>{appNames[app] ?? app}</span>
          </button>
        {/each}
      </div>
    {:else}
      <p class="muted">Apps you mention in your goal show up here. You connect accounts after the build.</p>
    {/if}
    <form
      class="add"
      onsubmit={(e) => {
        e.preventDefault();
        addLink('api');
      }}
    >
      <Input label="OpenAPI link" mono placeholder="https://api.example.com/openapi.json" bind:value={spec} error={specError || undefined} />
      <Button type="submit" disabled={!spec.trim()}>Add API</Button>
      <Button icon="upload" onclick={() => specPicker?.click()}>Upload spec</Button>
      <input bind:this={specPicker} type="file" hidden accept=".json,.yaml,.yml" onchange={() => picked(specPicker, true)} />
    </form>
    {#if apis.length}
      <ul class="list" aria-label="APIs">
        {#each apis as d (d.key)}
          <AttachmentRow draft={d} onrole={() => {}} onremove={() => remove(d.key)} />
        {/each}
      </ul>
    {/if}
  </section>
</div>

<footer class="bar">
  <span class="estimate">
    <Icon name="clock" size={18} />
    <span><strong>About 2 minutes</strong><span class="muted">You'll watch it build, then open it in the editor.</span></span>
  </span>
  <Button variant="ghost" href="/flows">Cancel</Button>
  <Button variant="primary" size="l" loading={starting} disabled={!canBuild} onclick={build}>Build it</Button>
</footer>

<style>
  .page { max-width: 760px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-5); }
  h1 { font: var(--t-h1); margin: 0; }
  header p { margin: var(--s-2) 0 0; color: var(--text-2); }
  .card {
    padding: var(--s-6); border-radius: var(--r-panel); background: var(--card-gradient); border: 1px solid var(--border);
    box-shadow: var(--shadow-pop-sm); display: flex; flex-direction: column; gap: var(--s-4);
  }
  h2 { display: flex; align-items: center; gap: var(--s-3); margin: 0; font-size: 18px; line-height: 26px; font-weight: 700; }
  .num {
    width: 28px; height: 28px; border-radius: 50%; display: inline-flex; align-items: center; justify-content: center;
    font-size: 13px; color: var(--primary); background: var(--primary-soft); border: 1px solid var(--primary-line);
  }
  .aside { margin-left: auto; font-size: 13px; font-weight: 400; color: var(--text-3); }
  .muted { font-size: 13px; color: var(--text-3); }
  .goal { position: relative; }
  .voice { position: absolute; right: 10px; bottom: 10px; }
  .examples { display: flex; flex-wrap: wrap; align-items: center; gap: var(--s-2); }
  .drop {
    display: flex; align-items: center; gap: var(--s-4); padding: var(--s-4) var(--s-5); border-radius: var(--r-card);
    border: 1px dashed var(--primary-line); background: var(--primary-soft);
  }
  .drop.dragging { border-style: solid; border-color: var(--primary); }
  .drop-icon {
    flex: 0 0 40px; height: 40px; border-radius: 10px; display: flex; align-items: center; justify-content: center;
    background: var(--surface); color: var(--primary); box-shadow: var(--shadow-tile);
  }
  .drop-text { flex: 1 1 auto; display: flex; flex-direction: column; gap: 2px; }
  .add { display: flex; align-items: flex-end; gap: var(--s-2); }
  .add :global(.field) { flex: 1 1 auto; }
  .note { display: flex; align-items: center; gap: 6px; margin: 0; font-size: 13px; color: var(--text-2); }
  .list {
    list-style: none; margin: 0; padding: 0; border: 1px solid var(--border); border-radius: var(--r-card);
    background: var(--surface); overflow: hidden;
  }
  .apps { display: flex; flex-wrap: wrap; gap: var(--s-4); }
  .app {
    display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 0; border: 0; background: none;
    font: 500 13px var(--font-sans); color: var(--text-2); cursor: pointer; border-radius: 8px;
  }
  .app.off { opacity: 0.4; text-decoration: line-through; }
  .app:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .bar {
    position: sticky; bottom: 0; display: flex; align-items: center; gap: var(--s-3);
    padding: var(--s-3) max(var(--s-6), calc((100% - 760px) / 2)); background: var(--surface); border-top: 1px solid var(--border);
  }
  .estimate { margin-right: auto; display: flex; align-items: center; gap: var(--s-3); color: var(--text-3); }
  .estimate > span { display: flex; flex-direction: column; font-size: 14px; color: var(--text); }
  @media (max-width: 767px) {
    .page { padding: var(--s-4); }
    .card { padding: var(--s-4); }
    .drop, .add { flex-wrap: wrap; }
    .estimate { display: none; }
    .bar { padding: var(--s-3) var(--s-4); justify-content: flex-end; }
  }
</style>
