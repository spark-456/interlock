<script lang="ts">
  // An API added to a build: where its spec comes from, how many endpoints became tools, and the risky ones skipped.
  import IconButton from './IconButton.svelte';
  import Icon from './Icon.svelte';

  let {
    name,
    source,
    endpoints,
    skipped = 0,
    onremove
  }: { name: string; source: string; endpoints?: number; skipped?: number; onremove?: () => void } = $props();
</script>

<article class="api" aria-label={name}>
  <span class="tile"><Icon name="api" size={18} /></span>
  <div class="text">
    <strong>{name}</strong>
    <span class="source">{source}</span>
    {#if endpoints !== undefined}
      <span class="stats">{endpoints} {endpoints === 1 ? 'endpoint' : 'endpoints'}{#if skipped} · <span class="skipped">{skipped} risky skipped</span>{/if}</span>
    {/if}
  </div>
  {#if onremove}<IconButton icon="x" label="Remove {name}" onclick={onremove} />{/if}
</article>

<style>
  .api { display: flex; align-items: flex-start; gap: var(--s-3); padding: var(--s-3) var(--s-4); border: 1px solid var(--border); border-radius: var(--r-card); background: var(--surface); }
  .tile { flex: none; display: grid; place-items: center; width: 32px; height: 32px; border-radius: var(--r-md); background: var(--cat-tool-tile); color: var(--cat-tool-icon); }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: var(--s-1); }
  .source { font: var(--t-code); color: var(--text-3); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .stats { font: var(--t-small); color: var(--text-2); }
  .skipped { color: var(--warning-text); }
</style>
