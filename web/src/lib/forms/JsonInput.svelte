<script lang="ts">
  // A JSON value edited as text: the draft stays as typed, and only valid JSON is saved.
  import type { JsonValue } from '$lib/api/types';
  import TextArea from '$lib/design/TextArea.svelte';
  import { readJson } from './schema';

  let { id, label, value, onchange, error, hint }: {
    id: string;
    label: string;
    value: JsonValue;
    onchange: (v: JsonValue) => void;
    error?: string;
    hint?: string;
  } = $props();

  // The draft starts from the saved value; later edits come from typing.
  // svelte-ignore state_referenced_locally
  let draft = $state(JSON.stringify(value ?? [], null, 2));
  let invalid = $state<string | null>(null);

  function edit(text: string) {
    draft = text;
    const read = readJson(text);
    invalid = 'error' in read ? read.error : null;
    if (!('error' in read)) onchange(read.value);
  }
</script>

<div class="json">
  <TextArea {id} {label} rows={6} spellcheck="false" bind:value={() => draft, edit} error={invalid ?? error} {hint} />
</div>

<style>
  .json :global(textarea) { font-family: var(--font-mono); font-size: 12px; }
</style>
