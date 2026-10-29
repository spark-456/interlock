<script lang="ts">
  // Drop files (or a link) here, or browse. The caller decides what to do with them.
  import type { Snippet } from 'svelte';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';

  let {
    title,
    hint,
    accept = '',
    onfiles,
    onlink,
    children
  }: { title: string; hint?: string; accept?: string; onfiles: (files: File[]) => void; onlink?: (url: string) => void; children?: Snippet } =
    $props();
  let dragging = $state(false);
  let picker: HTMLInputElement | undefined = $state();

  function drop(e: DragEvent) {
    e.preventDefault();
    dragging = false;
    const files = [...(e.dataTransfer?.files ?? [])];
    const uri = e.dataTransfer?.getData('text/uri-list');
    if (files.length) onfiles(files);
    else if (uri && onlink) onlink(uri);
  }
</script>

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
  <span class="icon"><Icon name="upload" size={20} /></span>
  <div class="text">
    <strong>{title}</strong>
    {#if hint}<span>{hint}</span>{/if}
    {@render children?.()}
  </div>
  <Button size="s" onclick={() => picker?.click()}>Browse files</Button>
  <input
    bind:this={picker}
    type="file"
    multiple
    hidden
    {accept}
    data-testid="file-input"
    onchange={() => {
      onfiles([...(picker?.files ?? [])]);
      if (picker) picker.value = '';
    }}
  />
</div>

<style>
  .drop {
    display: flex; align-items: center; gap: var(--s-4); padding: var(--s-4) var(--s-5); border-radius: var(--r-card);
    border: 1px dashed var(--primary-line); background: var(--primary-soft);
  }
  .dragging { border-style: solid; border-color: var(--primary); }
  .icon {
    flex: 0 0 40px; height: 40px; border-radius: var(--r-card); display: flex; align-items: center; justify-content: center;
    background: var(--surface); color: var(--primary); box-shadow: var(--shadow-tile);
  }
  .text { flex: 1 1 auto; display: flex; flex-direction: column; gap: var(--s-1); }
  .text span { font: var(--t-small); color: var(--text-2); }
  @media (max-width: 600px) { .drop { flex-wrap: wrap; } }
</style>
