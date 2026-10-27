<script lang="ts">
  // Clarifying questions (AR-2): pick an option or type an answer; the build continues once sent.
  import type { BuildQuestion } from '$lib/api/types';
  import Button from '$lib/design/Button.svelte';
  import Chip from '$lib/design/Chip.svelte';
  import Input from '$lib/design/Input.svelte';

  let { questions, onanswer }: { questions: BuildQuestion[]; onanswer: (answers: Record<string, string>) => Promise<void> } =
    $props();
  let answers = $state<Record<string, string>>({});
  let sending = $state(false);
  const complete = $derived(questions.every((q) => answers[q.id]?.trim()));

  async function send() {
    sending = true;
    try {
      await onanswer(answers);
    } finally {
      sending = false;
    }
  }
</script>

<section class="card" aria-label="Questions from Interlock" data-testid="questions">
  <h3>A few quick questions</h3>
  {#each questions as q (q.id)}
    <div class="q">
      <p>{q.text}</p>
      <div class="options">
        {#each q.options as o (o)}
          <Chip kind="filter" selected={answers[q.id] === o} onclick={() => (answers[q.id] = o)}>{o}</Chip>
        {/each}
      </div>
      <Input label="Or type your answer" bind:value={() => answers[q.id] ?? '', (v) => (answers[q.id] = v)} />
    </div>
  {/each}
  <Button variant="primary" loading={sending} disabled={!complete} onclick={send}>Continue building</Button>
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: 16px; padding: 20px; border-radius: var(--r-panel); background: var(--surface);
    border: 1px solid var(--border); box-shadow: var(--shadow-pop);
  }
  h3 { margin: 0; font-size: 16px; font-weight: 700; }
  .q { display: flex; flex-direction: column; gap: 8px; }
  p { margin: 0; font-size: 14px; font-weight: 600; }
  .options { display: flex; flex-wrap: wrap; gap: 6px; }
  .card :global(.btn) { align-self: flex-end; }
</style>
