<script lang="ts">
  // What a flow is: an automation (runs on a trigger) or an AI app (answers people).
  import type { FlowKind } from '$lib/api/types';
  import Icon from './Icon.svelte';

  let { kind }: { kind: FlowKind } = $props();
</script>

<span class="badge {kind}"><Icon name={kind === 'app' ? 'chat' : 'bolt'} size={12} />{kind === 'app' ? 'AI app' : 'Automation'}</span>

<style>
  .badge { display: inline-flex; align-items: center; gap: var(--s-1); padding: 0 var(--s-2); height: 22px; border-radius: var(--r-pill); font: var(--t-small); font-weight: 600; }
  .app { background: var(--cat-kb-tile); color: var(--cat-kb-icon); }
  .automation { background: var(--cat-trigger-tile); color: var(--cat-trigger-icon); }
</style>
