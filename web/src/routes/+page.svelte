<script lang="ts">
  // Landing (design S1 / M1): the pitch, the recorded build playing in the hero, how it works,
  // what you get, and the one-line integration.
  import BuiltWithStrip from '$lib/design/BuiltWithStrip.svelte';
  import Button from '$lib/design/Button.svelte';
  import Icon from '$lib/design/Icon.svelte';
  import Logo from '$lib/design/Logo.svelte';
  import PartnerSlot from '$lib/design/PartnerSlot.svelte';
  import Tabs from '$lib/design/Tabs.svelte';
  import { endpoints, quickStart } from './flows/[id]/publish/publish';

  const GITHUB = 'https://github.com/spark-456/interlock';
  const STEPS = [
    { title: 'Describe and drop in', text: 'Say what it should do. Add files, a website and an API spec. Mark anything that should be followed as rules.' },
    { title: 'Watch it build', text: 'The knowledge base, agents, prompts, tools and guardrails appear on a canvas. Tests run, and failures are fixed and re-run.' },
    { title: 'Connect anywhere', text: 'Use the OpenAI-compatible API, plug the MCP server into any client, or drop in the chat widget.' }
  ];
  const FEATURES = [
    { icon: 'db', title: 'Knowledge base', text: 'Your documents and website, indexed for search by meaning and by keyword, with dates so the newest policy wins.' },
    { icon: 'agent', title: 'Agents and prompts', text: 'A router and specialist agents, each with a written system prompt you can read and edit.' },
    { icon: 'wrench', title: 'Tools from your APIs', text: 'Endpoints become tools. Interlock picks the ones your goal needs and skips risky ones, like deletes.' },
    { icon: 'shield', title: 'Guardrails', text: 'Plain-language rules, checked on every answer and every tool call.' },
    { icon: 'tests', title: 'Tested before launch', text: 'Tests come from your documents and API. Failures are fixed automatically and run again.' },
    { icon: 'api', title: 'API, MCP and widget', text: 'An OpenAI-compatible API, an MCP server, a one-line chat widget and an integration guide.' }
  ];
  const WORKS_WITH = ['OpenAI SDK', 'LangChain', 'Vercel AI SDK', 'Claude Code', 'Cursor', 'VS Code'];
  const ends = endpoints('https://interlock.example', 'Larkspur Support');
  const code = quickStart(ends, 'Can I get a refund for last month?').filter((t) => ['js', 'python', 'curl'].includes(t.value));
  let tab = $state('js');
</script>

<svelte:head>
  <title>Interlock · Your AI backend, built from a sentence</title>
  <meta name="description" content="Describe the goal, drop in your documents and APIs. Interlock builds the knowledge base, agents, prompts, tools and tests, and hands you an API and an MCP server." />
</svelte:head>

<header class="nav">
  <a href="/" class="brand" aria-label="Interlock home"><Logo variant="lockup" size={28} /></a>
  <nav aria-label="Site">
    <a href="#how">How it works</a>
    <a href="/templates">Examples</a>
    <a href={GITHUB}>GitHub</a>
  </nav>
  <div class="nav-actions">
    <a class="signin" href="/api/auth/github/start">Sign in</a>
    <Button variant="primary" size="s" href="/demo">Try the demo</Button>
  </div>
</header>

<main class="landing">
  <section class="hero">
    <p class="eyebrow"><span>Goal in</span> AI backend out</p>
    <h1>Your AI backend, built from a sentence.</h1>
    <p class="lede">Describe the goal, drop in your documents and APIs. Interlock builds the knowledge base, agents, prompts, tools and tests, and hands you an API and an MCP server.</p>
    <div class="ctas">
      <Button variant="primary" href="/demo">Try the demo — no sign-in <Icon name="arrow-right" size={16} /></Button>
      <Button href={GITHUB}>View on GitHub</Button>
    </div>
    <ul class="checks">
      {#each ['About two minutes per build', 'Tested before it goes live', 'No code, no model picking'] as c (c)}<li><Icon name="check" size={14} /> {c}</li>{/each}
    </ul>
    <!-- Loaded after the first paint: the canvas library is most of the page's weight. -->
    {#await import('./HeroCanvas.svelte')}<div class="hero-slot"></div>{:then m}<m.default />{/await}
  </section>

  <section class="built">
    <BuiltWithStrip />
    <p>Runs on NVIDIA Nemotron open models through Nebius Token Factory. Websites are read with Tavily.</p>
  </section>

  <section id="how" class="block">
    <p class="kicker">How it works</p>
    <h2>From a goal to a working API in three steps</h2>
    <ol class="steps">
      {#each STEPS as s, i (s.title)}
        <li><span class="num">{i + 1}</span><h3>{s.title}</h3><p>{s.text}</p></li>
      {/each}
    </ol>
  </section>

  <section class="block">
    <p class="kicker">What you get</p>
    <h2>Everything a real assistant needs, built and tested for you</h2>
    <div class="features">
      {#each FEATURES as f (f.title)}
        <article><span class="tile"><Icon name={f.icon} size={18} /></span><h3>{f.title}</h3><p>{f.text}</p></article>
      {/each}
    </div>
  </section>

  <section class="works">
    <div><h2>Works with what you already use</h2><p>Any OpenAI SDK, your framework of choice, and your coding agent.</p></div>
    <ul>{#each WORKS_WITH as w (w)}<li><PartnerSlot name={w} height={28} /></li>{/each}</ul>
  </section>

  <section class="block connect">
    <div>
      <p class="kicker">Connect in one line</p>
      <h2>Change the base URL. Keep your code.</h2>
      <p>Your backend speaks the OpenAI API, so the SDK you already use works as is. Or give your coding agent the Interlock MCP and ask it to write the integration for you.</p>
      <ul class="checks col">
        {#each ['Streaming, plus endpoints to search and add documents', 'An MCP server for every backend', 'A ready INTERLOCK.md for your coding agent'] as c (c)}<li><Icon name="check" size={14} /> {c}</li>{/each}
      </ul>
    </div>
    <div class="code">
      <Tabs label="Language" tabs={code} bind:value={tab} />
      <pre><code>{code.find((c) => c.value === tab)?.code}</code></pre>
      <pre class="cc"><code><span>Claude Code</span> claude mcp add interlock {ends.platformMcp}</code></pre>
    </div>
  </section>

  <section class="cta">
    <Logo variant="app-icon" size={48} />
    <h2>Build your first backend in two minutes.</h2>
    <p>Open a ready-made demo instantly, or build your own with up to 20 pages and one API. No sign-in.</p>
    <Button href="/demo">Try the demo <Icon name="arrow-right" size={16} /></Button>
  </section>
</main>

<footer class="foot">
  <div class="row">
    <Logo variant="lockup" size={24} />
    <nav aria-label="Footer"><a href="/templates">Examples</a><a href={GITHUB}>GitHub</a></nav>
  </div>
  <div class="row small">
    <BuiltWithStrip />
    <span>Built for the Nebius × NVIDIA Global AI Hackathon 2026.</span>
  </div>
</footer>

<style>
  .nav {
    position: sticky; top: 0; z-index: 20; display: flex; align-items: center; gap: var(--s-6); padding: var(--s-3) var(--s-6);
    background: var(--surface); border-bottom: 1px solid var(--border);
  }
  .brand { display: inline-flex; }
  .nav nav { display: flex; gap: var(--s-5); }
  .nav a { color: var(--text-2); text-decoration: none; font-weight: 500; }
  .nav a:hover { color: var(--text); }
  .nav-actions { margin-left: auto; display: flex; align-items: center; gap: var(--s-4); }
  .landing { display: flex; flex-direction: column; }
  section { width: 100%; max-width: 1120px; margin: 0 auto; padding: 0 var(--s-6); box-sizing: border-box; }
  .hero { padding-top: var(--s-10); padding-bottom: var(--s-8); display: flex; flex-direction: column; gap: var(--s-5); }
  .eyebrow { margin: 0; align-self: flex-start; display: inline-flex; gap: var(--s-2); align-items: center; padding: var(--s-1) var(--s-2); border: 1px solid var(--border); border-radius: var(--r-pill); font: var(--t-small); color: var(--text-2); }
  .eyebrow span { padding: 0 var(--s-2); border-radius: var(--r-pill); background: var(--primary-soft); color: var(--primary); font-weight: 600; }
  h1 { margin: 0; max-width: 640px; font: var(--t-display); }
  .lede { margin: 0; max-width: 620px; color: var(--text-2); font: var(--t-body); }
  .ctas { display: flex; gap: var(--s-3); flex-wrap: wrap; }
  .checks { list-style: none; margin: 0 0 var(--s-5); padding: 0; display: flex; flex-wrap: wrap; gap: var(--s-2) var(--s-6); color: var(--text-2); font: var(--t-small); }
  .checks li { display: inline-flex; align-items: center; gap: var(--s-1); }
  .checks :global(svg) { color: var(--primary); }
  .hero-slot { height: 500px; border: 1px solid var(--border); border-radius: var(--r-panel); background: var(--surface-3); }
  .checks.col { flex-direction: column; margin: var(--s-4) 0 0; }
  .built { padding-top: var(--s-8); padding-bottom: var(--s-8); display: flex; flex-direction: column; align-items: center; gap: var(--s-3); text-align: center; }
  .built p { margin: 0; color: var(--text-3); font: var(--t-small); }
  .block { padding-top: var(--s-10); padding-bottom: var(--s-6); }
  .kicker { margin: 0 0 var(--s-2); color: var(--primary); font: var(--t-small); font-weight: 600; }
  h2 { margin: 0; font: var(--t-h1); max-width: 620px; }
  .steps { list-style: none; margin: var(--s-6) 0 0; padding: 0; display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--s-5); }
  .steps li, .features article {
    padding: var(--s-5); background: var(--card-gradient); border: 1px solid var(--border); border-radius: var(--r-panel); box-shadow: var(--shadow-pop-sm);
  }
  .num { display: inline-grid; place-items: center; width: 24px; height: 24px; border-radius: var(--r-card); background: var(--primary); color: var(--surface); font: var(--t-small); }
  h3 { margin: var(--s-3) 0 var(--s-2); font: var(--t-h3); }
  .steps p, .features p { margin: 0; color: var(--text-2); }
  .features { margin-top: var(--s-6); display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--s-5); }
  .tile { display: inline-grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-card); background: var(--primary-soft); color: var(--primary); }
  .works {
    margin-top: var(--s-10); display: flex; align-items: center; justify-content: space-between; gap: var(--s-6); flex-wrap: wrap;
    padding-top: var(--s-5); padding-bottom: var(--s-5); border: 1px solid var(--border); border-radius: var(--r-panel); max-width: 1072px;
  }
  .works h2 { font: var(--t-h3); }
  .works p { margin: var(--s-1) 0 0; color: var(--text-3); }
  .works ul { list-style: none; margin: 0; padding: 0; display: flex; gap: var(--s-4); flex-wrap: wrap; }
  .connect { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr); gap: var(--s-8); align-items: center; }
  .connect p { color: var(--text-2); }
  .code { display: flex; flex-direction: column; gap: var(--s-2); }
  pre { margin: 0; padding: var(--s-4); overflow-x: auto; border-radius: var(--r-card); background: var(--text); color: var(--surface); font: var(--t-code); }
  .cc span { margin-right: var(--s-2); color: var(--text-4); }
  .cta {
    margin: var(--s-10) auto; padding: var(--s-10) var(--s-6); max-width: 1072px; display: flex; flex-direction: column; align-items: center; gap: var(--s-3); text-align: center;
    border-radius: var(--r-panel); background: var(--primary); color: var(--surface);
  }
  .cta h2 { max-width: 480px; }
  .cta p { margin: 0; max-width: 460px; opacity: 0.85; }
  .foot { border-top: 1px solid var(--border); padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-4); max-width: 1120px; margin: 0 auto; width: 100%; box-sizing: border-box; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: var(--s-4); flex-wrap: wrap; }
  .row nav { display: flex; gap: var(--s-5); }
  .row a { color: var(--text-2); text-decoration: none; }
  .small { color: var(--text-3); font: var(--t-small); }
  @media (max-width: 1023px) {
    .steps, .features { grid-template-columns: 1fr 1fr; }
    .connect { grid-template-columns: minmax(0, 1fr); }
  }
  @media (max-width: 767px) {
    .nav { padding: var(--s-3) var(--s-4); }
    .nav nav, .signin { display: none; }
    section { padding: 0 var(--s-4); }
    .hero { padding-top: var(--s-8); }
    .steps, .features { grid-template-columns: 1fr; }
    .works, .cta { margin-left: var(--s-4); margin-right: var(--s-4); width: auto; }
  }
</style>
