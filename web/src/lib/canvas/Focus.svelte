<script lang="ts">
  // Brings the given nodes into view whenever the list changes (e.g. the steps an Ask proposal touches).
  import { untrack } from 'svelte';
  import { useSvelteFlow } from '@xyflow/svelte';

  let { ids }: { ids: string[] } = $props();
  const flow = useSvelteFlow();

  $effect(() => {
    const nodes = ids.map((id) => ({ id }));
    if (!nodes.length) return;
    // After a tick so new nodes are measured; untracked so only `ids` re-runs this.
    const t = setTimeout(() => untrack(() => void flow.fitView({ nodes, padding: 0.2, maxZoom: 1, duration: 300 })), 50);
    return () => clearTimeout(t);
  });
</script>
