<script lang="ts">
  type Option = { value: string; label: string };
  let { options, value = $bindable(''), label }: { options: Option[]; value?: string; label: string } = $props();
</script>

<div class="seg" role="group" aria-label={label}>
  {#each options as o (o.value)}
    <button type="button" aria-pressed={value === o.value} onclick={() => (value = o.value)}>{o.label}</button>
  {/each}
</div>

<style>
  .seg { display: inline-flex; padding: 3px; background: var(--surface-2); border-radius: var(--r-tile); }
  button {
    height: 30px; padding: 0 12px; border: 0; border-radius: var(--r-control); background: transparent;
    color: var(--text-2); font: 600 13px var(--font-sans); cursor: pointer;
  }
  button[aria-pressed='true'] { background: var(--surface); color: var(--text); box-shadow: var(--shadow-segment); }
  button:focus-visible { outline: none; box-shadow: var(--focus-ring); }
</style>
