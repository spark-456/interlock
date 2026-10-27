<script lang="ts">
  // Ask Interlock (ED-9 / AR-8): a change request in plain words. The page shows the returned diff
  // on the canvas; this bar holds the request, then the summary with Accept and Discard.
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import IconButton from '$lib/design/IconButton.svelte';

  let {
    pending,
    summary,
    onask,
    onaccept,
    ondiscard,
    onclose
  }: {
    pending: boolean;
    /** Summary of the proposal on screen, or null before one arrives. */
    summary: string | null;
    onask: (message: string) => void;
    onaccept: () => void;
    ondiscard: () => void;
    onclose: () => void;
  } = $props();

  let message = $state('');
</script>

<div class="ask" role="region" aria-label="Ask Interlock" data-testid="ask-bar">
  <Icon name="sparkle" size={18} color="var(--primary)" />
  {#if summary}
    <p class="summary" aria-live="polite"><strong>Proposed:</strong> {summary}</p>
    <Button size="s" onclick={ondiscard}>Discard</Button>
    <Button size="s" variant="primary" onclick={onaccept}>Accept</Button>
  {:else}
    <form
      onsubmit={(e) => {
        e.preventDefault();
        if (message.trim()) onask(message.trim());
      }}
    >
      <input
        aria-label="Describe a change"
        placeholder="Describe a change, e.g. “Ask me in Slack before posting”"
        bind:value={message}
        disabled={pending}
      />
      <Button size="s" variant="primary" type="submit" loading={pending} disabled={!message.trim()}>Propose</Button>
    </form>
    <IconButton icon="x" label="Close" onclick={onclose} />
  {/if}
</div>

<style>
  .ask {
    display: flex; align-items: center; gap: 10px; width: min(720px, calc(100vw - 32px)); padding: 8px 8px 8px 14px;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-card); box-shadow: var(--shadow-md);
  }
  form { flex: 1; display: flex; align-items: center; gap: 8px; min-width: 0; }
  input { flex: 1; min-width: 0; height: 32px; border: 0; outline: 0; background: transparent; font: inherit; font-size: 14px; color: var(--text); }
  .summary { flex: 1; min-width: 0; font-size: 14px; line-height: 20px; color: var(--text-2); }
  .summary strong { color: var(--text); font-weight: 600; }
</style>
