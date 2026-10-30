// Run with Node 22: node deploy/smoke.mjs https://your-interlock-host
const base = new URL(process.argv[2] ?? 'http://127.0.0.1:8080');

async function request(path) {
  const response = await fetch(new URL(path, base), {
    signal: AbortSignal.timeout(15_000),
  });
  if (!response.ok) throw new Error(`GET ${path}: HTTP ${response.status}`);
  return { response, body: await response.json() };
}

try {
  const health = await request('/healthz');
  if (health.body.ok !== true) throw new Error('/healthz did not report ready');

  const catalog = await request('/api/nodes');
  if (!Array.isArray(catalog.body.nodes) || catalog.body.nodes.length === 0) {
    throw new Error('/api/nodes returned an empty catalog');
  }

  // A public deploy has no sign-in-free workspace: data needs a signed-in session.
  const anonymous = await fetch(new URL('/api/flows', base), { signal: AbortSignal.timeout(15_000) });
  const local = anonymous.ok;
  if (!local && anonymous.status !== 401) throw new Error(`/api/flows without a session: HTTP ${anonymous.status}`);

  console.log(
    `Smoke passed: health, ${catalog.body.nodes.length} nodes, ` +
      (local ? 'local workspace ON (never on a public host)' : 'sign-in required'),
  );
} catch (error) {
  console.error(`Smoke failed: ${error.message}`);
  process.exitCode = 1;
}
