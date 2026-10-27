<script lang="ts">
  let { checked = $bindable(false), label, disabled = false }: { checked?: boolean; label: string; disabled?: boolean } = $props();
</script>

<label class="checkbox" class:disabled>
  <input type="checkbox" bind:checked {disabled} />
  <span class="box" aria-hidden="true">
    <svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-7" /></svg>
  </span>
  <span>{label}</span>
</label>

<style>
  .checkbox { display: inline-flex; align-items: center; gap: var(--s-2); cursor: pointer; position: relative; }
  input { position: absolute; opacity: 0; width: 1px; height: 1px; }
  .box {
    flex: none; width: 18px; height: 18px; border-radius: var(--r-sm); border: 1px solid var(--border-strong);
    background: var(--surface); display: grid; place-items: center; transition: background var(--dur-pill) ease;
  }
  svg { width: 14px; height: 14px; fill: none; stroke: var(--surface); stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; opacity: 0; }
  input:checked + .box { background: var(--primary); border-color: var(--primary); box-shadow: var(--shadow-check); }
  input:checked + .box svg { opacity: 1; }
  input:focus-visible + .box { box-shadow: var(--focus-ring); }
  .disabled { opacity: .5; cursor: not-allowed; }
</style>
