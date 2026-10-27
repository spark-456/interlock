<script lang="ts">
  let { checked = $bindable(false), label, disabled = false }: { checked?: boolean; label: string; disabled?: boolean } = $props();
</script>

<label class="toggle" class:disabled>
  <input type="checkbox" role="switch" bind:checked {disabled} />
  <span class="track" aria-hidden="true"><span class="knob"></span></span>
  <span>{label}</span>
</label>

<style>
  .toggle { display: inline-flex; align-items: center; gap: 10px; font-size: 14px; cursor: pointer; position: relative; }
  input { position: absolute; opacity: 0; width: 1px; height: 1px; }
  .track {
    width: 40px; height: 24px; border-radius: var(--r-pill); background: var(--border); position: relative; flex: none;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, .08); transition: background var(--dur-pill) ease;
  }
  .knob {
    position: absolute; left: 2px; top: 2px; width: 20px; height: 20px; border-radius: 50%; background: var(--surface);
    box-shadow: 0 1px 3px rgba(16, 24, 40, .2); transition: transform var(--dur-pill) ease;
  }
  input:checked + .track { background: var(--primary); box-shadow: inset 0 1px 2px rgba(0, 0, 0, .15); }
  input:checked + .track .knob { transform: translateX(16px); }
  input:focus-visible + .track { box-shadow: var(--focus-ring); }
  .disabled { opacity: .5; cursor: not-allowed; }
</style>
