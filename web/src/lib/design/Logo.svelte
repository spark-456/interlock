<script lang="ts">
  // Interlock mark (two interlocking rounded squares), lockup (mark + wordmark) and app icon,
  // from design/interlock/brand/*.svg. Colours are the brand's own (primary on white).
  let { variant = 'mark', size = 28 }: { variant?: 'mark' | 'lockup' | 'app-icon'; size?: number } = $props();
  const uid = $props.id();
  const clip = `il-logo-${uid}`;
</script>

{#if variant === 'app-icon'}
  <svg width={size} height={size} viewBox="0 0 48 48" role="img" aria-label="Interlock">
    <rect width="48" height="48" rx="11" class="fill-primary" />
    <defs><clipPath id={clip}><rect x="24" y="0" width="24" height="24" /></clipPath></defs>
    <g transform="translate(9 9) scale(0.625)" fill="none">
      <rect x="6.5" y="6.5" width="21" height="21" rx="8" class="st-white" stroke-width="5.75" />
      <rect x="20.5" y="20.5" width="21" height="21" rx="8" class="st-primary" stroke-width="13" />
      <rect x="20.5" y="20.5" width="21" height="21" rx="8" class="st-white" stroke-width="5.75" />
      <g clip-path="url(#{clip})">
        <rect x="6.5" y="6.5" width="21" height="21" rx="8" class="st-primary" stroke-width="13" />
        <rect x="6.5" y="6.5" width="21" height="21" rx="8" class="st-white" stroke-width="5.75" />
      </g>
    </g>
  </svg>
{:else}
  <span class="logo">
    <svg width={size} height={size} viewBox="0 0 48 48" role="img" aria-label="Interlock">
      <defs><clipPath id={clip}><rect x="24" y="0" width="24" height="24" /></clipPath></defs>
      <g fill="none">
        <rect x="6.5" y="6.5" width="21" height="21" rx="8" class="st-primary" stroke-width="5.75" />
        <rect x="20.5" y="20.5" width="21" height="21" rx="8" class="st-white" stroke-width="13" />
        <rect x="20.5" y="20.5" width="21" height="21" rx="8" class="st-primary" stroke-width="5.75" />
        <g clip-path="url(#{clip})">
          <rect x="6.5" y="6.5" width="21" height="21" rx="8" class="st-white" stroke-width="13" />
          <rect x="6.5" y="6.5" width="21" height="21" rx="8" class="st-primary" stroke-width="5.75" />
        </g>
      </g>
    </svg>
    {#if variant === 'lockup'}<span class="word" style:font-size="{Math.round(size * 0.68)}px">Interlock</span>{/if}
  </span>
{/if}

<style>
  .logo { display: inline-flex; align-items: center; gap: 10px; color: var(--text); }
  .word { font-weight: 700; letter-spacing: -0.02em; line-height: 1; }
  .fill-primary { fill: var(--primary); }
  .st-primary { stroke: var(--primary); }
  .st-white { stroke: var(--surface); }
</style>
