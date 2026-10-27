<script lang="ts">
  // One attachment: what it is, Knowledge / Instructions, remove. Errors replace the detail line.
  import type { AttachmentRole } from '$lib/api/types';
  import Icon from '$lib/design/Icon.svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import SegmentedControl from '$lib/design/SegmentedControl.svelte';
  import Spinner from '$lib/design/Spinner.svelte';
  import type { Draft } from './create';

  let { draft, onrole, onremove }: { draft: Draft; onrole: (r: AttachmentRole) => void; onremove: () => void } = $props();
  const ICONS = { link: 'globe', file: 'docs', audio: 'mic', api: 'plug' } as const;
</script>

<li class="row" class:error={!!draft.error}>
  <span class="tile"><Icon name={ICONS[draft.kind]} size={18} /></span>
  <div class="text">
    <div class="name">{draft.name}</div>
    <div class="detail">
      {#if draft.uploading}<Spinner size={10} /> Uploading…{:else}{draft.error ?? draft.detail}{/if}
    </div>
  </div>
  {#if !draft.error && draft.kind !== 'api'}
    <SegmentedControl
      label="Use {draft.name} as"
      options={[{ value: 'knowledge', label: 'Knowledge' }, { value: 'instructions', label: 'Instructions' }]}
      bind:value={() => draft.role, (v) => onrole(v as AttachmentRole)}
    />
  {/if}
  <IconButton icon="x" label="Remove {draft.name}" onclick={onremove} />
</li>

<style>
  .row { display: flex; align-items: center; gap: 12px; padding: 10px 12px; }
  .row + :global(.row) { border-top: 1px solid var(--surface-2); }
  .row.error { background: var(--error-soft); }
  .tile {
    flex: 0 0 32px; width: 32px; height: 32px; border-radius: 9px; border: 1px solid var(--border);
    display: flex; align-items: center; justify-content: center; color: var(--text-2); background: var(--surface);
  }
  .text { flex: 1 1 auto; min-width: 0; }
  .name { font-size: 14px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .detail { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-3); }
  .error .detail { color: var(--error-text); }
  @media (max-width: 600px) { .row { flex-wrap: wrap; } .text { flex-basis: calc(100% - 90px); } }
</style>
