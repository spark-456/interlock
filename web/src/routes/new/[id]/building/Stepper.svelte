<script lang="ts">
  // Desktop stepper: seven build stages with done / active / to-do markers and times.
  import { STAGES, testsRun, type BuildState } from '$lib/canvas/buildState';
  import Icon from '$lib/design/Icon.svelte';
  import { clock } from '$lib/design/format';

  let { build, now }: { build: BuildState; now: number } = $props();

  function time(key: string, status: string | undefined): string {
    const st = build.stages[key];
    if (status === 'done' && st?.start !== undefined && st.end !== undefined) return clock(st.end - st.start);
    if (status === 'active') {
      if (key === 'test') {
        const r = testsRun(build);
        if (r) return `${r.run} / ${r.total}`;
      }
      return st?.start !== undefined ? clock(now - st.start) : '';
    }
    return '';
  }
</script>

<ol class="steps" aria-label="Build steps">
  {#each STAGES as s (s.key)}
    {@const status = build.stages[s.key]?.status ?? (build.status === 'ready' ? 'done' : 'pending')}
    <li class={status} aria-current={status === 'active' ? 'step' : undefined}>
      {#if status === 'done'}
        <span class="mark done"><Icon name="check" size={12} strokeWidth={3.2} color="var(--surface)" /></span>
      {:else if status === 'active'}
        <span class="mark active"><svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="6" fill="none" class="trk" stroke-width="2" /><path d="M8 2a6 6 0 0 1 6 6" fill="none" class="arc" stroke-width="2" stroke-linecap="round" /></svg></span>
      {:else if status === 'failed'}
        <span class="mark failed"><Icon name="x" size={12} strokeWidth={3} color="var(--surface)" /></span>
      {:else}
        <span class="mark todo"></span>
      {/if}
      <span class="name">{s.label}</span>
      <span class="time">{time(s.key, status)}</span>
    </li>
  {/each}
</ol>

<style>
  .steps { list-style: none; margin: 0; padding: 16px 24px 8px; display: flex; flex-direction: column; gap: 2px; }
  li { display: flex; align-items: center; gap: 12px; height: 34px; }
  .mark { flex: 0 0 22px; width: 22px; height: 22px; border-radius: 50%; display: flex; align-items: center; justify-content: center; }
  .mark.done { background: var(--success); box-shadow: var(--shadow-check); }
  .mark.active { background: var(--primary-soft); }
  .mark.failed { background: var(--error); }
  .mark.todo { border: 1.5px dashed var(--border-strong); }
  .trk { stroke: var(--primary-line); }
  .arc { stroke: var(--primary); transform-origin: 8px 8px; animation: spin .9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .name { flex: 1 1 auto; font-size: 14px; font-weight: 500; color: var(--text); }
  .active .name { font-weight: 700; }
  .pending .name { color: var(--text-4); }
  .time { font: 400 12px var(--font-mono); color: var(--text-3); }
</style>
