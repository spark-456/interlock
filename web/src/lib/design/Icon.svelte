<script lang="ts">
  import { icons, isIcon } from './icons';

  let {
    name,
    size = 20,
    color = 'currentColor',
    strokeWidth = 2,
    label
  }: { name: string; size?: number; color?: string; strokeWidth?: number; label?: string } = $props();

  const body = $derived(isIcon(name) ? icons[name] : icons.box);
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke={color}
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  role={label ? 'img' : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : 'true'}
>
  <!-- eslint-disable-next-line svelte/no-at-html-tags -- static paths from icons.ts -->
  {@html body}
</svg>
