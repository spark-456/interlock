<script lang="ts">
  // Chat widget card: the one-line script tag and a still preview of the widget on a page.
  import CopyField from '$lib/design/CopyField.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import type { Endpoints } from './publish';

  let { endpoints, appName, greeting }: { endpoints: Endpoints; appName: string; greeting: string } = $props();
</script>

<section class="card" aria-labelledby="widget-title">
  <header>
    <span class="tile"><Icon name="chat" size={18} /></span>
    <div><h2 id="widget-title">Chat widget</h2><p>Paste one line into your site</p></div>
  </header>
  <CopyField label="Script tag" value={endpoints.widget} />
  <div>
    <span class="label">Live preview</span>
    <div class="preview" aria-hidden="true">
      <span class="dots"><i></i><i></i><i></i></span>
      <span class="line w60"></span><span class="line w80"></span><span class="line w40"></span>
      <div class="bubble"><strong>{appName}</strong><span>{greeting}</span></div>
      <span class="launch"><Icon name="chat" size={18} /></span>
    </div>
  </div>
</section>

<style>
  .card {
    display: flex; flex-direction: column; gap: var(--s-4); padding: var(--s-5);
    background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  header { display: flex; gap: var(--s-3); align-items: center; }
  .tile { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-card); background: var(--cat-source-tile); color: var(--cat-source-icon); }
  h2 { font: var(--t-h3); margin: 0; }
  header p { margin: 0; color: var(--text-3); font: var(--t-small); }
  .label { font: var(--t-small); font-weight: 600; color: var(--text-2); }
  .preview {
    position: relative; height: 200px; margin-top: var(--s-2); padding: var(--s-3); display: flex; flex-direction: column; gap: var(--s-2);
    border: 1px solid var(--border); border-radius: var(--r-card); background: var(--surface);
  }
  .dots { display: flex; gap: var(--s-1); margin-bottom: var(--s-2); }
  .dots i { width: 6px; height: 6px; border-radius: 50%; background: var(--border-strong); }
  .line { height: 6px; border-radius: 3px; background: var(--surface-4); }
  .w40 { width: 40%; } .w60 { width: 60%; } .w80 { width: 80%; }
  .bubble {
    position: absolute; right: var(--s-3); bottom: 56px; width: 70%; display: flex; flex-direction: column; overflow: hidden;
    border-radius: var(--r-card); background: var(--surface); box-shadow: var(--shadow-md); font: var(--t-small);
  }
  .bubble strong { padding: var(--s-2) var(--s-3); background: var(--primary); color: var(--surface); }
  .bubble span { padding: var(--s-2) var(--s-3); color: var(--text-2); }
  .launch {
    position: absolute; right: var(--s-3); bottom: var(--s-3); display: grid; place-items: center; width: 36px; height: 36px;
    border-radius: 50%; background: var(--primary); color: var(--surface);
  }
</style>
