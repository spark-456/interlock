<script lang="ts">
  // One attached file, link or memo: what it is, Knowledge / Instructions, remove. An error replaces the detail.
  import IconButton from './IconButton.svelte';
  import Icon from './Icon.svelte';
  import SegmentedControl from './SegmentedControl.svelte';
  import Spinner from './Spinner.svelte';

  type Role = 'knowledge' | 'instructions';
  let {
    icon = 'docs',
    name,
    detail = '',
    error,
    busy = false,
    role = $bindable(),
    onremove
  }: { icon?: string; name: string; detail?: string; error?: string; busy?: boolean; role?: Role; onremove?: () => void } = $props();
</script>

<li class="row" class:error={!!error}>
  <span class="tile"><Icon name={icon} size={18} /></span>
  <div class="text">
    <div class="name">{name}</div>
    <div class="detail">{#if busy}<Spinner size={10} /> Uploading…{:else}{error ?? detail}{/if}</div>
  </div>
  {#if !error && role}
    <SegmentedControl
      label="Use {name} as"
      options={[{ value: 'knowledge', label: 'Knowledge' }, { value: 'instructions', label: 'Instructions' }]}
      bind:value={() => role ?? 'knowledge', (v) => (role = v as Role)}
    />
  {/if}
  {#if onremove}<IconButton icon="x" label="Remove {name}" onclick={onremove} />{/if}
</li>

<style>
  .row { display: flex; align-items: center; gap: var(--s-3); padding: var(--s-2) var(--s-3); list-style: none; }
  .row + :global(.row) { border-top: 1px solid var(--surface-2); }
  .row.error { background: var(--error-soft); }
  .tile {
    flex: 0 0 32px; height: 32px; border-radius: var(--r-md); border: 1px solid var(--border);
    display: flex; align-items: center; justify-content: center; color: var(--text-2); background: var(--surface);
  }
  .text { flex: 1 1 auto; min-width: 0; }
  .name { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .detail { display: flex; align-items: center; gap: var(--s-1); font: var(--t-small); color: var(--text-3); }
  .error .detail { color: var(--error-text); }
  @media (max-width: 600px) { .row { flex-wrap: wrap; } .text { flex-basis: calc(100% - 90px); } }
</style>
