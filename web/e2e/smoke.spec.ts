import { expect, test } from '@playwright/test';

// Fixture mode smoke: every PRD §7.3 screen renders, and recorded streams replay.
const screens = [
  '/', '/new', '/new/bld_fa/building', '/flows', '/flows/flw_promos', '/flows/flw_promos/runs/run_fa',
  '/flows/flw_promos/publish', '/flows/flw_promos/chat', '/templates', '/connections', '/approvals', '/settings', '/demo'
];

for (const path of screens) {
  test(`renders ${path}`, async ({ page }) => {
    await page.goto(path);
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('h1').first()).toBeVisible();
  });
}

test('run stream replays to the end', async ({ page }) => {
  await page.goto('/flows/flw_promos/runs/run_fa');
  await expect(page.getByTestId('stream-status')).toContainText('done', { timeout: 30_000 });
});

test('F-A build stream replays to ready', async ({ page }) => {
  await page.goto('/new/bld_fa/building?speed=60');
  await expect(page.getByRole('link', { name: 'Open editor' }).first()).toBeVisible({ timeout: 30_000 });
});
