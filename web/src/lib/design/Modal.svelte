<script lang="ts">
  // Modal on the native <dialog> (focus trap, Escape, backdrop for free).
  import type { Snippet } from 'svelte';
  import IconButton from './IconButton.svelte';

  let { open = $bindable(false), title, children, footer }: {
    open?: boolean; title: string; children: Snippet; footer?: Snippet;
  } = $props();
  let dialog: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  });
</script>

<dialog bind:this={dialog} onclose={() => (open = false)} aria-labelledby="modal-title">
  <header>
    <h2 id="modal-title">{title}</h2>
    <IconButton icon="x" label="Close" onclick={() => (open = false)} />
  </header>
  <div class="body">{@render children()}</div>
  {#if footer}<footer>{@render footer()}</footer>{/if}
</dialog>

<style>
  dialog {
    width: min(560px, calc(100vw - 32px)); padding: 0; border: 1px solid var(--border); border-radius: var(--r-panel);
    background: var(--surface); color: var(--text); box-shadow: var(--shadow-sheet);
  }
  dialog::backdrop { background: rgba(16, 24, 40, .4); }
  header { display: flex; align-items: center; justify-content: space-between; padding: var(--s-5) var(--s-6) var(--s-3); }
  h2 { font: var(--t-h3); font-weight: 700; }
  .body { padding: 0 var(--s-6) var(--s-6); }
  footer { display: flex; justify-content: flex-end; gap: var(--s-2); padding: var(--s-4) var(--s-6); border-top: 1px solid var(--border); background: var(--surface-3); }
</style>
