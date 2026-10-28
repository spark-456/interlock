import { expect, test } from '@playwright/test';

// S7 flows list and the templates gallery, in fixture mode.

test('lists flows with filters and search', async ({ page }) => {
  await page.goto('/flows');
  const card = page.getByRole('article').filter({ hasText: 'Weekly product promos' });
  await expect(card).toContainText('Live');
  await expect(card.getByRole('img', { name: /runs in the last 14 days/ })).toBeVisible();

  const filters = page.getByRole('group', { name: 'Filter flows' });
  await filters.getByRole('button', { name: /Off/ }).click();
  await expect(card).toBeHidden();
  await page.getByLabel('Search flows').fill('shopify');
  await expect(page.getByText('No flows match.')).toBeVisible();
  await page.getByLabel('Search flows').fill('');
  await filters.getByRole('button', { name: /All/ }).click();
  await page.getByLabel('Search flows').fill('shopify');
  await expect(card).toBeVisible();

  await card.getByRole('link', { name: 'Weekly product promos' }).click();
  await expect(page).toHaveURL(/\/flows\/flw_promos$/);
});

test('an Off flow offers a test run that starts in the editor', async ({ page }) => {
  await page.goto('/flows');
  await expect(page.getByText('New flows start off')).toBeVisible();
  const off = page.getByRole('article').filter({ hasText: 'Larkspur Support' });
  await expect(page.getByRole('article').filter({ hasText: 'Weekly product promos' }).getByRole('link', { name: /Test run/ })).toHaveCount(0);
  await off.getByRole('link', { name: 'Test run Larkspur Support' }).click();
  await expect(page).toHaveURL(/\/flows\/flw_larkspur$/);
  await expect(page.getByText(/^Test run/).first()).toBeVisible({ timeout: 30_000 });
});

test('uses a template and opens it in the editor', async ({ page }) => {
  await page.goto('/templates');
  await expect(page.getByRole('article')).toHaveCount(10);
  const promos = page.getByRole('article').filter({ has: page.getByRole('heading', { name: 'Weekly product promos' }) });
  await promos.getByRole('button', { name: 'Use template' }).click();
  await expect(page).toHaveURL(/\/flows\/flw_promos$/);
});
