<script lang="ts">
  // T00 placeholder: replays the build stream and logs every event to the console.
  import { onDestroy } from 'svelte';
  import { page } from '$app/state';
  import Placeholder from '$lib/design/Placeholder.svelte';
  import { buildEvents } from '$lib/api';

  let count = $state(0);
  let last = $state('');
  let done = $state(false);
  const sub = buildEvents(page.params.id ?? '', (e) => {
    console.log('[build]', e.seq, e.type, e);
    count += 1;
    last = e.type;
  }, { onEnd: () => (done = true) });
  onDestroy(() => sub.close());
</script>

<Placeholder title="Building" task="W04" route="/new/[id]/building">
  <p data-testid="stream-status">{count} events · last: {last || '—'}{done ? ' · done' : ''}</p>
</Placeholder>
