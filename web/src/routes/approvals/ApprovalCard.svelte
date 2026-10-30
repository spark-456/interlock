<script lang="ts">
  // One approval: what the flow wants to do, an editable payload, and Approve / Reject. Used by the
  // approvals inbox and the run view.
  import { api } from '$lib/api';
  import type { Approval, Decision, JsonValue } from '$lib/api/types';
  import Button from '$lib/design/Button.svelte';
  import Input from '$lib/design/Input.svelte';
  import StatusPill from '$lib/design/StatusPill.svelte';
  import TextArea from '$lib/design/TextArea.svelte';
  import { toast } from '$lib/design/toasts.svelte';

  import { editableText, withText } from './approval';

  let { approval, flowName, ondecided }: { approval: Approval; flowName?: string; ondecided?: (d: Decision) => void } = $props();

  // The text that will be sent when there is one; otherwise the whole payload as JSON.
  const plain = $derived(editableText(approval.payload));
  const original = $derived(plain ? plain.text : JSON.stringify(approval.payload, null, 2));
  let edited = $state<string | null>(null);
  let comment = $state('');
  let busy = $state<Decision | null>(null);
  const text = $derived(edited ?? original);
  const invalid = $derived.by(() => {
    if (plain) return undefined;
    try {
      JSON.parse(text);
      return undefined;
    } catch {
      return 'This isn’t valid JSON.';
    }
  });
  const changed = (): JsonValue | null => {
    if (edited === null || edited === original) return null;
    return plain ? withText(approval.payload, plain, edited) : JSON.parse(edited);
  };
  const pending = $derived(approval.status === 'pending');
  const outcome = $derived(
    approval.status === 'approved' ? 'Approved' : approval.status === 'rejected' ? 'Rejected' : 'Expired'
  );

  async function decide(decision: Decision) {
    busy = decision;
    try {
      await api.decide(approval.id, { decision, payload: changed(), comment: comment.trim() || null });
      ondecided?.(decision);
    } catch (e) {
      toast(e instanceof Error ? e.message : 'Could not send the decision.', 'error');
    } finally {
      busy = null;
    }
  }
</script>

<article class="card" aria-label={approval.title}>
  <header>
    <div>
      <h3>{approval.title}</h3>
      {#if flowName}<a href="/flows/{approval.flowId}/runs/{approval.runId}">{flowName}</a>{/if}
    </div>
    {#if pending}
      <StatusPill tone="warning" label="Waiting" />
    {:else}
      <StatusPill tone={approval.status === 'approved' ? 'ready' : 'failed'} label={outcome} />
    {/if}
  </header>
  {#if approval.message}<p>{approval.message}</p>{/if}
  {#if pending}
    {#if approval.payload !== null}
      <div class:json={!plain}>
        <TextArea label="What will be sent (you can edit it)" rows={6} value={text} oninput={(e: Event) => (edited = (e.currentTarget as HTMLTextAreaElement).value)} error={invalid} />
      </div>
    {/if}
    <Input label="Comment (optional)" bind:value={comment} />
    <div class="actions">
      <Button variant="danger" loading={busy === 'reject'} disabled={!!busy} onclick={() => decide('reject')}>Reject</Button>
      <Button variant="primary" icon="check" loading={busy === 'approve'} disabled={!!busy || !!invalid} onclick={() => decide('approve')}>Approve</Button>
    </div>
  {:else if approval.decidedBy}
    <p class="by">{outcome} by {approval.decidedBy}</p>
  {/if}
</article>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-3); padding: var(--s-4) var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; justify-content: space-between; align-items: flex-start; gap: var(--s-3); }
  h3 { font: var(--t-h3); margin: 0; }
  header a { color: var(--text-3); font: var(--t-small); }
  p { margin: 0; color: var(--text-2); }
  .by { color: var(--text-3); font: var(--t-small); }
  .actions { display: flex; justify-content: flex-end; gap: var(--s-2); }
  .json :global(textarea) { font: var(--t-small); font-family: var(--font-mono); }
</style>
