<script lang="ts" module>
  export type PillTone = 'ready' | 'waiting' | 'building' | 'fixed' | 'warning' | 'failed' | 'live';
</script>

<script lang="ts">
  // Status is always text plus colour. Tones match the node states in DESIGN.md.
  import Icon from './Icon.svelte';
  import Spinner from './Spinner.svelte';

  let { tone, label, size = 24 }: { tone: PillTone; label: string; size?: 24 | 26 } = $props();
</script>

<span class="pill {tone}" style:height="{size}px">
  {#if tone === 'building'}<Spinner size={12} />
  {:else if tone === 'fixed'}<Icon name="check" size={12} strokeWidth={3} />
  {:else}<span class="dot"></span>{/if}
  {label}
</span>

<style>
  .pill {
    display: inline-flex; align-items: center; gap: 6px; padding: 0 9px; border-radius: var(--r-pill);
    font-size: 12px; font-weight: 600; white-space: nowrap; flex: none;
    transition: background var(--dur-pill) ease, color var(--dur-pill) ease;
  }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
  .ready, .live { background: var(--pill-ready-bg); color: var(--pill-ready-fg); }
  .ready .dot, .live .dot { background: var(--pill-ready-dot); }
  .waiting { background: var(--pill-waiting-bg); color: var(--pill-waiting-fg); }
  .waiting .dot { background: var(--pill-waiting-dot); }
  .building, .fixed { background: var(--pill-building-bg); color: var(--pill-building-fg); }
  .warning { background: var(--pill-warning-bg); color: var(--pill-warning-fg); }
  .warning .dot { background: var(--pill-warning-dot); }
  .failed { background: var(--pill-failed-bg); color: var(--pill-failed-fg); }
  .failed .dot { background: var(--pill-failed-dot); }
</style>
