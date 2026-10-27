<script lang="ts">
  // Which connected account each credential uses (ED-3). Only connection ids reach the flow;
  // tokens never come to the browser. Missing or expired connections offer "Connect".
  import type { ConnectionSummary, CredentialSpec } from '$lib/api/types';
  import Button from '$lib/design/Button.svelte';
  import Select from '$lib/design/Select.svelte';

  let {
    specs,
    connections,
    value,
    onchange
  }: {
    specs: CredentialSpec[];
    connections: ConnectionSummary[];
    value: { [key in string]?: string };
    onchange: (v: { [key in string]?: string }) => void;
  } = $props();

  const usable = (app: string) => connections.filter((c) => c.app === app && c.status === 'connected' && c.id);
  const connectHref = (s: CredentialSpec) => (s.kind === 'oAuth2' ? `/api/oauth/${s.app}/start` : '/connections');
  const expired = (app: string) => connections.some((c) => c.app === app && c.status === 'expired');
</script>

<div class="creds">
  {#each specs as s (s.key)}
    {@const options = usable(s.app)}
    {#if options.length}
      <Select
        label={s.optional ? `${s.label} (optional)` : `${s.label} *`}
        options={[
          ...(s.optional || !value[s.key] ? [{ value: '', label: 'Choose an account' }] : []),
          ...options.map((c) => ({ value: c.id ?? '', label: c.account ?? c.label }))
        ]}
        bind:value={() => value[s.key] ?? '', (v) => onchange({ ...value, [s.key]: v || undefined })}
      />
    {:else}
      <div class="missing" data-testid="connect-{s.key}">
        <div>
          <div class="label">{s.label}{s.optional ? ' (optional)' : ''}</div>
          <div class="hint">{expired(s.app) ? 'The connection expired.' : 'Not connected yet.'}</div>
        </div>
        <Button size="s" icon="plug" href={connectHref(s)}>{expired(s.app) ? 'Reconnect' : 'Connect'}</Button>
      </div>
    {/if}
  {/each}
</div>

<style>
  .creds { display: flex; flex-direction: column; gap: 12px; }
  .missing {
    display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 12px;
    border: 1px dashed var(--border-strong); border-radius: var(--r-card);
  }
  .label { font-size: 13px; font-weight: 600; }
  .hint { font-size: 12px; color: var(--text-3); }
</style>
