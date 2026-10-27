import { expect, test } from '@playwright/test';

// Demo picker, settings and the error, offline and cap-reached states, in fixture mode.

test('the demo opens a seeded template and starts a build of your own', async ({ page }) => {
  await page.goto('/demo');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Try Interlock without an account.');
  await expect(page.getByRole('complementary', { name: 'Main' })).toHaveCount(0);
  const card = page.getByRole('article').filter({ hasText: 'Weekly product promos' });
  await expect(card).toContainText('Opens instantly');
  await card.getByRole('button', { name: 'Open' }).click();
  await expect(page).toHaveURL(/\/flows\/flw_promos$/);

  await page.goto('/demo');
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page).toHaveURL(/\/new$/);
});

test('settings show the workspace and manage API keys', async ({ page }) => {
  await page.goto('/settings');
  await expect(page.getByRole('region', { name: 'Workspace' })).toContainText('Dana Ruiz');
  const keys = page.getByRole('list', { name: 'API keys' });
  await expect(keys.getByRole('listitem')).toHaveCount(2);
  await page.getByLabel('New key name').fill('CI');
  await page.getByRole('button', { name: 'Create key' }).click();
  await expect(page.getByText('il_live_5Hq8Lw2Zp9Vt3f9aK1mX')).toBeVisible();
  await keys.getByRole('listitem').filter({ hasText: 'Website widget' }).getByRole('button', { name: 'Revoke' }).click();
  await expect(keys.getByRole('listitem').filter({ hasText: 'Website widget' })).toContainText('Revoked');
});

test('unknown pages show the not-found state', async ({ page }) => {
  await page.goto('/no-such-page');
  await expect(page.getByRole('heading', { name: 'This page doesn’t exist' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Go to your flows' })).toHaveAttribute('href', '/flows');
});

test('going offline shows the offline banner', async ({ page, context }) => {
  await page.goto('/flows');
  await expect(page.getByRole('heading', { name: 'Flows' })).toBeVisible();
  await context.setOffline(true);
  await expect(page.getByText("You're offline", { exact: false })).toBeVisible();
  await context.setOffline(false);
  await expect(page.getByText("You're offline", { exact: false })).toHaveCount(0);
});

test('a run that hits its cap says so', async ({ page }) => {
  await page.goto('/flows/flw_promos/runs/run_cap');
  await expect(page.getByRole('alert').filter({ hasText: 'Spending cap reached (run)' })).toBeVisible({ timeout: 15_000 });
});
