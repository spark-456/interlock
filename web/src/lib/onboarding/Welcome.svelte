<script lang="ts">
  // First-run welcome (Q35): what Interlock is and the two kinds of flow. Any way of closing it counts as seen.
  import { goto } from '$app/navigation';
  import Button from '$lib/design/Button.svelte';
  import KindBadge from '$lib/design/KindBadge.svelte';
  import Modal from '$lib/design/Modal.svelte';
  import { onboarding, remember, startTour } from './state.svelte';

  function close() {
    onboarding.welcomeOpen = false;
    remember({ welcomed: true });
  }
</script>

<Modal bind:open={() => onboarding.welcomeOpen, (open) => !open && close()} title="Welcome to Interlock">
  <p class="lead">
    Describe what you want in a sentence. Interlock picks the steps, connects your apps, tests the result and hands you
    something that runs.
  </p>
  <ul class="kinds" aria-label="Two kinds of flow">
    <li>
      <KindBadge kind="automation" />
      <p>Runs on its own when something happens, like a schedule, a new email or a webhook, and does the work in your apps.</p>
      <p class="eg">“Every morning, summarise new support emails and post the summary in Slack.”</p>
    </li>
    <li>
      <KindBadge kind="app" />
      <p>Answers people in chat, over an API or from Claude Code and Codex, using your documents and tools.</p>
      <p class="eg">“A support assistant that answers from our help center and opens tickets.”</p>
    </li>
  </ul>
  <p class="muted">The tour takes about a minute and points at each screen. Open it again any time from Help or Settings.</p>
  {#snippet footer()}
    <div class="actions">
      <Button variant="ghost" onclick={close}>Skip for now</Button>
      <Button onclick={() => (close(), goto('/new'))}>Build my first flow</Button>
      <Button variant="primary" icon="play" onclick={startTour}>Take the tour</Button>
    </div>
  {/snippet}
</Modal>

<style>
  .lead { margin: 0 0 var(--s-4); color: var(--text-2); }
  .kinds { list-style: none; margin: 0 0 var(--s-4); padding: 0; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--s-3); }
  .kinds li {
    display: flex; flex-direction: column; align-items: flex-start; gap: var(--s-2); padding: var(--s-4);
    border: 1px solid var(--border); border-radius: var(--r-card); background: var(--card-gradient); box-shadow: var(--shadow-pop-sm);
  }
  .kinds p { margin: 0; font: var(--t-small); color: var(--text-2); }
  .kinds .eg { color: var(--text-3); font-style: italic; }
  .muted { margin: 0; font: var(--t-small); color: var(--text-3); }
  .actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: var(--s-2); }
  @media (max-width: 767px) { .kinds { grid-template-columns: 1fr; } }
</style>
