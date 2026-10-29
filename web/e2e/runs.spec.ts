import { expect, test } from '@playwright/test';

// Run view, approvals inbox and connections, in fixture mode.

test('a run shows its steps, clips and the approval decision', async ({ page }) => {
  await page.goto('/flows/flw_promos/runs/run_fa');
  const details = page.getByRole('complementary', { name: 'Run details' });
  await expect(details.getByText('Approved by Priya (Slack)')).toBeVisible({ timeout: 30_000 });
  await expect(details.getByRole('region', { name: 'Clips' }).getByRole('figure')).toHaveCount(3);
  const steps = details.getByRole('region', { name: 'Steps' }).getByRole('listitem');
  await expect(steps).toHaveCount(18);
  await expect(steps.filter({ hasText: 'Cut for each platform' })).toContainText('9 items');
});

test('approves from the inbox with an edited payload', async ({ page }) => {
  await page.goto('/approvals');
  await expect(page.getByText('1 waiting for you.')).toBeVisible();
  const card = page.getByRole('article', { name: 'Post 3 promo videos' });
  const payload = card.getByLabel('What will be sent (you can edit it)');
  await payload.fill('{ "posts": [');
  await expect(card.getByRole('button', { name: 'Approve' })).toBeDisabled();
  await payload.fill('{ "posts": [] }');
  const sent = page.waitForRequest((r) => r.url().includes('/approvals/apr_fa/decide'));
  await card.getByRole('button', { name: 'Approve' }).click();
  await sent;
  await expect(page.getByText('Nothing is waiting for you.')).toBeVisible();
  await expect(page.getByRole('region', { name: 'Recent decisions' })).toContainText('Approved by you');
});

test('lists apps with their state and opens the connect drawer', async ({ page }) => {
  await page.goto('/connections');
  const apps = page.getByRole('list', { name: 'Apps' });
  await expect(apps.getByRole('button').first()).toHaveAccessibleName('Google, Connected');
  await apps.getByRole('button', { name: 'Discord, Expired' }).click();
  const drawer = page.getByRole('dialog', { name: 'Discord' });
  await expect(drawer).toContainText('rather than a sign-in');
  await drawer.getByRole('button', { name: 'Close' }).click();
  await apps.getByRole('button', { name: 'Slack, Connected' }).click();
  await expect(page.getByRole('dialog', { name: 'Slack' }).getByLabel('Bot token')).toBeVisible();
});

test('saves a bot token for a key-based app', async ({ page }) => {
  await page.goto('/connections');
  await page.getByRole('list', { name: 'Apps' }).getByRole('button', { name: 'Telegram, Not connected' }).click();
  const drawer = page.getByRole('dialog', { name: 'Telegram' });
  await drawer.getByLabel('Bot token').fill('123456:ABC');
  const sent = page.waitForRequest((r) => r.url().includes('connections.post'));
  await drawer.getByRole('button', { name: 'Save', exact: true }).click();
  expect(JSON.parse((await sent).postData() ?? '{}')).toBeTruthy();
  await expect(page.getByText('Telegram is connected.')).toBeVisible();
});

test('Q12 mobile run cards open their step details', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/flows/flw_promos/runs/run_fa?speed=120');
  const card = page.getByTestId('node-list').getByRole('button', { name: /Cut for each platform.*Open details/ });
  await expect(card).toBeVisible({ timeout: 15_000 });
  await card.click();
  const detail = page.getByRole('region', { name: 'Steps' }).locator('[data-step="n_resize"]');
  await expect(detail).toHaveAttribute('data-selected', 'true');
  await expect(detail).toBeFocused();
});

