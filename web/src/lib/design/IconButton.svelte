<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';

  type Props = HTMLButtonAttributes & {
    icon: string;
    label: string;
    variant?: 'secondary' | 'ghost';
    size?: number;
    pressed?: boolean;
  };
  let { icon, label, variant = 'ghost', size = 32, pressed, type = 'button', ...rest }: Props = $props();
</script>

<button class="ib {variant}" class:pressed {type} aria-label={label} title={label} aria-pressed={pressed} style:--size="{size}px" {...rest}>
  <Icon name={icon} size={16} />
</button>

<style>
  .ib {
    width: var(--size); height: var(--size); display: inline-flex; align-items: center; justify-content: center;
    border-radius: var(--r-control); border: 0; background: transparent; color: var(--text-2); cursor: pointer; flex: none;
  }
  .secondary { background: var(--surface); border: 1px solid var(--border-strong); box-shadow: var(--shadow-button-secondary); }
  .ib:hover { background: var(--surface-2); }
  .ib.pressed { background: var(--surface-2); color: var(--text); }
  .ib:active { background: var(--surface-4); }
  .ib:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .ib:disabled { color: var(--text-4); cursor: not-allowed; background: transparent; }
</style>
