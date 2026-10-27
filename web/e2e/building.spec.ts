import { expect, test } from '@playwright/test';

// S3 Building in fixture mode: the recorded Larkspur build replays to "ready", and the canvas
// lets you drag a node with its edges following.
const URL = '/new/bld_larkspur/building?speed=60';

test('replays the recorded build to ready', async ({ page }) => {
  await page.goto(URL);
  await expect(page.getByRole('heading', { name: /Larkspur Support/ })).toBeVisible();
  await expect(page.getByTestId('node-n_kb')).toBeVisible();
  await expect(page.getByTestId('fix-card')).toBeVisible({ timeout: 30_000 });
  await expect(page.getByRole('link', { name: 'Open editor' }).first()).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId('build-cost')).toHaveText('$0.084');
});

test('freezes at a moment with ?at=', async ({ page }) => {
  await page.goto('/new/bld_larkspur/building?speed=60&at=88');
  await expect(page.getByTestId('node-n_tests')).toContainText('21 / 30', { timeout: 30_000 });
  await expect(page.getByTestId('build-timer')).toContainText('1:28');
});

test('dragging a node moves its edges with it', async ({ page }) => {
  await page.goto('/new/bld_larkspur/building?speed=60&at=50');
  const card = page.getByTestId('node-n_kb');
  await expect(card).toBeVisible({ timeout: 30_000 });
  const edge = page.locator('.svelte-flow__edge[data-id="e_web_kb"] path').first();
  await expect(edge).toBeAttached();
  const before = await edge.getAttribute('d');
  const box = (await card.boundingBox())!;

  await page.mouse.move(box.x + box.width / 2, box.y + 20);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 60, box.y + 80, { steps: 8 });
  await page.mouse.move(box.x + box.width / 2 + 120, box.y + 140, { steps: 8 });
  await page.mouse.up();

  const after = (await card.boundingBox())!;
  expect(after.x - box.x).toBeGreaterThan(80);
  expect(after.y - box.y).toBeGreaterThan(80);
  const moved = await edge.getAttribute('d');
  expect(moved).not.toBe(before);
  // The edge now ends at the dragged card's left edge (in screen space, within a few px).
  const end = await edge.evaluate((p: SVGPathElement) => {
    const pt = p.getPointAtLength(p.getTotalLength());
    const m = p.getScreenCTM()!;
    return { x: pt.x * m.a + m.e, y: pt.y * m.d + m.f };
  });
  expect(Math.abs(end.x - after.x)).toBeLessThan(12);
  expect(end.y).toBeGreaterThan(after.y);
  expect(end.y).toBeLessThan(after.y + after.height);
});

test('mobile shows the grouped list instead of the canvas', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/new/bld_larkspur/building?speed=60&at=88');
  await expect(page.getByTestId('node-list')).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId('node-canvas')).toHaveCount(0);
  await expect(page.getByRole('navigation', { name: 'Tabs' })).toBeVisible();
});
