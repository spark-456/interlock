// Run with Node 22: node deploy/smoke.mjs https://your-interlock-host
const base = new URL(process.argv[2] ?? 'http://127.0.0.1:8080');

async function request(path, { method = 'GET', cookie } = {}) {
  const response = await fetch(new URL(path, base), {
    method,
    headers: cookie ? { Cookie: cookie } : {},
    signal: AbortSignal.timeout(15_000),
  });
  if (!response.ok) throw new Error(`${method} ${path}: HTTP ${response.status}`);
  return { response, body: await response.json() };
}

try {
  const health = await request('/healthz');
  if (health.body.ok !== true) throw new Error('/healthz did not report ready');

  const catalog = await request('/api/nodes');
  if (!Array.isArray(catalog.body.nodes) || catalog.body.nodes.length === 0) {
    throw new Error('/api/nodes returned an empty catalog');
  }

  const demo = await request('/api/demo/session', { method: 'POST' });
  if (demo.body.demo !== true) throw new Error('demo session was not created');
  const cookie = demo.response.headers.get('set-cookie')?.split(';', 1)[0];
  if (!cookie) throw new Error('demo session did not set a cookie');

  const flows = await request('/api/flows', { cookie });
  const template = flows.body.find((flow) => flow.lastRun?.replay === true);
  if (!template) throw new Error('no seeded template has a recorded run');
  const run = await request(`/api/runs/${encodeURIComponent(template.lastRun.id)}`, { cookie });
  if (run.body.run?.status !== 'succeeded' || run.body.run?.replay !== true) {
    throw new Error('seeded template run is not a successful replay');
  }

  console.log(`Smoke passed: health, ${catalog.body.nodes.length} nodes, demo session, ${template.name} replay`);
} catch (error) {
  console.error(`Smoke failed: ${error.message}`);
  process.exitCode = 1;
}
