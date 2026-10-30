<script lang="ts">
  // First-run onboarding (Q35): the welcome on a person's first visit, the guided tour and the Help dialog.
  // Progress is remembered per user in the browser.
  import { beforeNavigate } from '$app/navigation';
  import type { Me } from '$lib/api';
  import Button from '$lib/design/Button.svelte';
  import Modal from '$lib/design/Modal.svelte';
  import Checklist from './Checklist.svelte';
  import Tour from './Tour.svelte';
  import Welcome from './Welcome.svelte';
  import { onboarding, signIn, startTour } from './state.svelte';

  let { me }: { me: Me | null } = $props();

  $effect(() => {
    if (me) signIn(me.user?.id ?? me.workspaceId);
  });

  // A checklist link leaves the Help dialog behind otherwise. Only navigations that start while it is
  // open close it: one the tour started earlier may finish after Help was opened.
  beforeNavigate(() => (onboarding.helpOpen = false));
</script>

<Welcome />
<Tour />
<Modal bind:open={() => onboarding.helpOpen, (open) => (onboarding.helpOpen = open)} title="Getting started">
  {#if onboarding.helpOpen}
    <p class="lead">Four steps to your first working flow. Each one ticks itself off as you go.</p>
    <Checklist />
  {/if}
  {#snippet footer()}
    <div class="actions">
      <Button variant="ghost" onclick={() => ((onboarding.helpOpen = false), (onboarding.welcomeOpen = true))}>Show the welcome</Button>
      <Button variant="primary" icon="play" onclick={startTour}>Take the tour</Button>
    </div>
  {/snippet}
</Modal>

<style>
  .lead { margin: 0 0 var(--s-4); color: var(--text-2); }
  .actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: var(--s-2); }
</style>
