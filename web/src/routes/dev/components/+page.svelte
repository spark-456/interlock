<script lang="ts">
  // /dev/components: every design component in every state (design SP1), for review and visual checks.
  import ApiCard from '$lib/design/ApiCard.svelte';
  import AppTile from '$lib/design/AppTile.svelte';
  import Banner from '$lib/design/Banner.svelte';
  import BottomSheet from '$lib/design/BottomSheet.svelte';
  import BuiltWithStrip from '$lib/design/BuiltWithStrip.svelte';
  import Button from '$lib/design/Button.svelte';
  import Checkbox from '$lib/design/Checkbox.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import CodeTabs from '$lib/design/CodeTabs.svelte';
  import CopyField from '$lib/design/CopyField.svelte';
  import DropZone from '$lib/design/DropZone.svelte';
  import EmptyState from '$lib/design/EmptyState.svelte';
  import FileRow from '$lib/design/FileRow.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import Input from '$lib/design/Input.svelte';
  import KindBadge from '$lib/design/KindBadge.svelte';
  import Logo from '$lib/design/Logo.svelte';
  import Menu from '$lib/design/Menu.svelte';
  import Modal from '$lib/design/Modal.svelte';
  import PartnerSlot from '$lib/design/PartnerSlot.svelte';
  import SegmentedControl from '$lib/design/SegmentedControl.svelte';
  import Select from '$lib/design/Select.svelte';
  import Skeleton from '$lib/design/Skeleton.svelte';
  import Spinner from '$lib/design/Spinner.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import Tabs from '$lib/design/Tabs.svelte';
  import TextArea from '$lib/design/TextArea.svelte';
  import Toggle from '$lib/design/Toggle.svelte';
  import Tooltip from '$lib/design/Tooltip.svelte';
  import { toast } from '$lib/design/toasts.svelte';

  const VARIANTS = ['primary', 'secondary', 'ghost', 'danger'] as const;
  const TONES = ['waiting', 'building', 'ready', 'fixed', 'warning', 'failed', 'live'] as const;
  let segment = $state('knowledge');
  let tab = $state('prompt');
  let on = $state(true);
  let ticked = $state(true);
  let role = $state<'knowledge' | 'instructions'>('knowledge');
  let modal = $state(false);
  let sheet = $state(false);
  let dropped = $state('');
  let last = $state('');
</script>

<svelte:head><title>Components · Interlock</title></svelte:head>

<div class="page">
  <h1>Components</h1>
  <p class="muted">Every design component in every state. Matches <code>design/interlock/screens/SpecV3.png</code>.</p>

  <section aria-labelledby="c-buttons">
    <h2 id="c-buttons">Button · IconButton</h2>
    {#each VARIANTS as v (v)}
      <div class="row">
        <Button variant={v}>{v}</Button>
        <Button variant={v} disabled>Disabled</Button>
        <Button variant={v} loading>Building…</Button>
        <Button variant={v} icon="plus" size="s">Small</Button>
      </div>
    {/each}
    <div class="row">
      <IconButton icon="copy" label="Copy" />
      <IconButton icon="eye" label="Show" variant="secondary" />
      <IconButton icon="list" label="List view" pressed />
    </div>
  </section>

  <section aria-labelledby="c-inputs">
    <h2 id="c-inputs">Input · TextArea · Select</h2>
    <div class="grid">
      <Input label="Backend name" value="Larkspur Support" />
      <Input label="OpenAPI link" mono value="api.larkspur.example/openapi" error="We couldn't read that spec. Check the link or upload the file." />
      <Input label="Disabled" disabled value="Read only" />
      <Input label="With a hint" hint="Shown under the field." />
      <TextArea label="Instructions" value="Answer from the help center and cite sources." />
      <Select label="Model role" options={[{ value: 'agent', label: 'Agent' }, { value: 'fast', label: 'Fast' }]} value="agent" />
    </div>
  </section>

  <section aria-labelledby="c-choice">
    <h2 id="c-choice">SegmentedControl · Toggle · Checkbox · Tabs</h2>
    <div class="row">
      <SegmentedControl label="Use as" options={[{ value: 'knowledge', label: 'Knowledge' }, { value: 'instructions', label: 'Instructions' }]} bind:value={segment} />
      <Toggle label="Let Interlock choose endpoints" bind:checked={on} />
      <Toggle label="Disabled" disabled />
      <Checkbox label="Include sources" bind:checked={ticked} />
      <Checkbox label="Unchecked" />
      <Checkbox label="Disabled" disabled />
    </div>
    <Tabs label="Panel" tabs={[{ value: 'prompt', label: 'Prompt' }, { value: 'knowledge', label: 'Knowledge' }, { value: 'tools', label: 'Tools' }]} bind:value={tab} />
  </section>

  <section aria-labelledby="c-status">
    <h2 id="c-status">Chip · StatusPill · KindBadge</h2>
    <div class="row">
      <Chip>Support assistant</Chip>
      <Chip kind="filter" selected onclick={() => {}}>Live <span class="muted">4</span></Chip>
      <Chip kind="filter" onclick={() => {}}>Building</Chip>
      <Chip kind="source" icon="docs">Refund policy 2026 · p. 2</Chip>
      <Chip kind="tool">lookup_customer</Chip>
      <Chip kind="diff-old">14 days</Chip>
      <Chip kind="diff-new">30 days</Chip>
    </div>
    <div class="row">
      {#each TONES as t (t)}<StatusPill tone={t} label={t} />{/each}
      <KindBadge kind="app" />
      <KindBadge kind="automation" />
    </div>
  </section>

  <section aria-labelledby="c-overlay">
    <h2 id="c-overlay">Tooltip · Menu · Modal · BottomSheet · Toast · Banner</h2>
    <div class="row">
      <Tooltip text="Copies the key"><Button icon="copy">Hover or focus me</Button></Tooltip>
      <Menu label="More actions" items={[
        { label: 'Duplicate', icon: 'copy', onselect: () => (last = 'Duplicate') },
        { label: 'Rename', icon: 'pen', onselect: () => (last = 'Rename') },
        { label: 'Delete', icon: 'trash', danger: true, onselect: () => (last = 'Delete') }
      ]} />
      {#if last}<span class="muted">Picked: {last}</span>{/if}
      <Button onclick={() => (modal = true)}>Open modal</Button>
      <Button onclick={() => (sheet = true)}>Open bottom sheet</Button>
      <Button onclick={() => toast('Saved.', 'success')}>Toast</Button>
      <Button onclick={() => toast('Could not save.', 'error')}>Error toast</Button>
    </div>
    <Banner kind="offline">You're offline. Saving, builds and runs work again once you reconnect.</Banner>
    <Banner kind="cap">Spending cap reached (run). The run stopped instead of spending more.</Banner>
    <Modal bind:open={modal} title="A modal">
      <p>Focus stays inside; Escape closes it.</p>
      {#snippet footer()}<Button variant="primary" onclick={() => (modal = false)}>Done</Button>{/snippet}
    </Modal>
    <BottomSheet bind:open={sheet} title="A bottom sheet"><p>Used on phones for details and actions.</p></BottomSheet>
  </section>

  <section aria-labelledby="c-data">
    <h2 id="c-data">CopyField · CodeTabs</h2>
    <div class="grid">
      <CopyField label="Masked" value="il_live_5Hq8Lw2Zp9Vt3f9a" mode="masked" />
      <CopyField label="Reveal once" value="il_live_7c1d0b52e9a43f9a" mode="reveal-once" />
      <CopyField label="Plain" value="https://interlock.example/v1" />
    </div>
    <CodeTabs snippets={[
      { value: 'curl', label: 'cURL', code: 'curl https://interlock.example/v1/models -H "Authorization: Bearer $INTERLOCK_KEY"' },
      { value: 'js', label: 'JavaScript', code: 'const res = await fetch("https://interlock.example/v1/models");' }
    ]} />
  </section>

  <section aria-labelledby="c-files">
    <h2 id="c-files">DropZone · FileRow · ApiCard</h2>
    <DropZone title="Drop files or paste a link" hint="PDF, DOCX, Markdown, TXT, CSV or a voice memo." onfiles={(f) => (dropped = f.map((x) => x.name).join(', '))} />
    {#if dropped}<p class="muted">Got: {dropped}</p>{/if}
    <ul class="list">
      <FileRow name="Refund policy 2026.pdf" detail="2 pages · 84 KB" bind:role onremove={() => {}} />
      <FileRow icon="mic" name="Voice memo.m4a" detail="0:15" role="instructions" />
      <FileRow name="Contracts.zip" busy />
      <FileRow name="Huge scan.pdf" error="Files must be 50 MB or smaller." onremove={() => {}} />
    </ul>
    <div class="grid">
      <ApiCard name="Larkspur Admin API" source="https://api.larkspur.example/openapi.json" endpoints={11} skipped={2} onremove={() => {}} />
      <ApiCard name="Uploaded spec" source="invoices.yaml" />
    </div>
  </section>

  <section aria-labelledby="c-feedback">
    <h2 id="c-feedback">EmptyState · Skeleton · Spinner</h2>
    <EmptyState title="Your first flow is two minutes away." text="Describe a goal, drop in a few documents, and Interlock does the rest.">
      {#snippet actions()}<Button variant="primary">New flow</Button><Button>Browse templates</Button>{/snippet}
    </EmptyState>
    <div class="grid">
      <Skeleton lines={3} />
      <div class="row"><Spinner size={16} /><span class="muted">Loading…</span></div>
    </div>
  </section>

  <section aria-labelledby="c-brand">
    <h2 id="c-brand">Brand · AppTile · PartnerSlot</h2>
    <div class="row">
      <Logo /><Logo variant="lockup" /><Logo variant="app-icon" size={40} />
      <AppTile name="Gmail" /><AppTile name="Slack" size={24} />
      <PartnerSlot name="NVIDIA" /><PartnerSlot name="Nebius" />
    </div>
    <BuiltWithStrip />
  </section>
</div>

<style>
  .page { max-width: 1040px; margin: 0 auto; padding: var(--s-8) var(--s-6) var(--s-10); display: flex; flex-direction: column; gap: var(--s-6); }
  h1 { font: var(--t-h1); margin: 0; }
  section { display: flex; flex-direction: column; gap: var(--s-4); padding: var(--s-5); border: 1px solid var(--border); border-radius: var(--r-panel); background: var(--surface); }
  h2 { font: var(--t-h3); margin: 0; }
  .row { display: flex; flex-wrap: wrap; align-items: center; gap: var(--s-3); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: var(--s-4); align-items: start; }
  .list { margin: 0; padding: 0; border: 1px solid var(--border); border-radius: var(--r-card); overflow: hidden; }
  .muted { color: var(--text-3); }
  @media (max-width: 767px) { .page { padding: var(--s-4); } }
</style>
