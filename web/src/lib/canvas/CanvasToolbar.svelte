<script lang="ts" module>
  export interface EditTools {
    onundo: () => void;
    onredo: () => void;
    ontidy: () => void;
    canUndo: boolean;
    canRedo: boolean;
  }
</script>

<script lang="ts">
  // Canvas toolbar (bottom-left): zoom −, %, +, fit, minimap toggle, list view; when editing also
  // undo, redo and tidy.
  import { useSvelteFlow, useViewport } from '@xyflow/svelte';
  import IconButton from '$lib/design/IconButton.svelte';
  import { MAX_ZOOM, MIN_ZOOM } from './geometry';

  let { minimap = $bindable(true), onlist, edit }: { minimap?: boolean; onlist?: () => void; edit?: EditTools } = $props();
  const flow = useSvelteFlow();
  const viewport = useViewport();
  const pct = $derived(Math.round(viewport.current.zoom * 100));
</script>

<div class="toolbar" role="toolbar" aria-label="Canvas">
  <IconButton icon="minus" label="Zoom out" disabled={viewport.current.zoom <= MIN_ZOOM + 0.001} onclick={() => flow.zoomOut()} />
  <span class="pct" aria-live="polite">{pct}%</span>
  <IconButton icon="plus" label="Zoom in" disabled={viewport.current.zoom >= MAX_ZOOM - 0.001} onclick={() => flow.zoomIn()} />
  <span class="sep" aria-hidden="true"></span>
  <IconButton icon="fit" label="Fit to screen" onclick={() => flow.fitView({ padding: 0.06, duration: 200 })} />
  <IconButton icon="minimap" label="Toggle minimap" pressed={minimap} onclick={() => (minimap = !minimap)} />
  {#if onlist}<IconButton icon="list" label="List view" onclick={onlist} />{/if}
  {#if edit}
    <span class="sep" aria-hidden="true"></span>
    <IconButton icon="undo" label="Undo" disabled={!edit.canUndo} onclick={edit.onundo} />
    <IconButton icon="redo" label="Redo" disabled={!edit.canRedo} onclick={edit.onredo} />
    <IconButton icon="tidy" label="Tidy layout" onclick={edit.ontidy} />
  {/if}
</div>

<style>
  .toolbar {
    display: flex; align-items: center; gap: 2px; padding: 4px; background: var(--surface);
    border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-md);
  }
  .pct { min-width: 44px; text-align: center; font: 500 12px var(--font-mono); color: var(--control-text); }
  .sep { width: 1px; height: 20px; background: var(--border); margin: 0 2px; }
</style>
