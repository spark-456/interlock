<script lang="ts">
  // Button: primary | secondary | ghost | danger; S 32 / M 40 / L 48. Loading keeps the width.
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import Spinner from './Spinner.svelte';

  type Props = HTMLButtonAttributes & {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 's' | 'm' | 'l';
    loading?: boolean;
    icon?: string;
    href?: string;
    /** With `href`: download the link's target under this file name. */
    download?: string;
    children: Snippet;
  };
  let { variant = 'secondary', size = 'm', loading = false, icon, href, download, children, disabled, type = 'button', ...rest }: Props = $props();
</script>

{#if href && !disabled}
  <a class="btn {variant} {size}" {href} {download}>
    {#if icon}<Icon name={icon} size={16} />{/if}
    {@render children()}
  </a>
{:else}
  <button class="btn {variant} {size}" class:loading {type} disabled={disabled || loading} aria-busy={loading || undefined} {...rest}>
    {#if loading}<span class="spin"><Spinner size={14} /></span>{:else if icon}<Icon name={icon} size={16} />{/if}
    <span class="label">{@render children()}</span>
  </button>
{/if}

<style>
  .btn {
    display: inline-flex; align-items: center; justify-content: center; gap: var(--s-2);
    border-radius: var(--r-control); border: 0; font-weight: 600; text-decoration: none; cursor: pointer;
    white-space: nowrap; transition: background var(--dur-pill) ease, box-shadow var(--dur-pill) ease, transform 80ms ease;
  }
  .s { height: 32px; padding: 0 12px; font-size: 13px; }
  .m { height: 40px; padding: 0 16px; font-size: 14px; }
  .l { height: 48px; padding: 0 22px; font-size: 15px; }
  .primary { background: var(--primary); color: var(--surface); box-shadow: var(--shadow-button-primary); }
  .primary:hover { background: var(--primary-hover); }
  .secondary { background: var(--surface); color: var(--control-text); border: 1px solid var(--border-strong); box-shadow: var(--shadow-button-secondary); }
  .secondary:hover { background: var(--surface-3); }
  .ghost { background: transparent; color: var(--control-text); }
  .ghost:hover { background: var(--surface-2); }
  .danger { background: var(--error); color: var(--surface); box-shadow: var(--shadow-button-danger); }
  .danger:hover { background: var(--error-text); }
  .btn:focus-visible { outline: none; box-shadow: var(--focus-ring); }
  .btn:active:not(:disabled) { transform: translateY(1px); box-shadow: var(--shadow-pressed); }
  .secondary:active:not(:disabled) { background: var(--surface-2); }
  .ghost:active:not(:disabled) { background: var(--surface-4); box-shadow: none; }
  .btn:disabled { cursor: not-allowed; color: var(--text-4); box-shadow: none; }
  .primary:disabled, .danger:disabled { background: var(--border); }
  .secondary:disabled { background: var(--surface); border-color: var(--border); }
  .ghost:disabled { background: transparent; }
  .btn.loading:disabled { cursor: progress; }
  .primary.loading, .danger.loading { color: var(--surface); }
  .primary.loading { background: var(--primary); box-shadow: var(--shadow-button-primary); }
  .danger.loading { background: var(--error); }
  .secondary.loading, .ghost.loading { color: var(--control-text); }
  .spin { display: inline-flex; color: currentColor; }
</style>
