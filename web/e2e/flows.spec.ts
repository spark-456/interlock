import { expect, test } from '@playwright/test';

// S7 flows list and the templates gallery, in fixture mode.

test('lists flows with filters and search', async ({ page }) => {
  await page.goto('/flows');
  const card = page.getByRole('article').filter({ hasText: 'Weekly product promos' });
  await expect(card).toContainText('Live');
  await expect(card.getByRole('img', { name: /runs in the last 14 days/ })).toBeVisible();

  const filters = page.getByRole('group', { name: 'Filter flows' });
  await filters.getByRole('button', { name: /Off/ }).click();
  await expect(page.getByText('No flows match.')).toBeVisible();
  await filters.getByRole('button', { name: /All/ }).click();
  await page.getByLabel('Search flows').fill('shopify');
  await expect(card).toBeVisible();

  await card.getByRole('link', { name: 'Weekly product promos' }).click();
  await expect(page).toHaveURL(/\/flows\/flw_promos$/);
});

test('uses a template and opens it in the editor', async ({ page }) => {
  await page.goto('/templates');
  await expect(page.getByRole('heading', { name: 'Weekly product promos' })).toBeVisible();
  await page.getByRole('button', { name: 'Use template' }).click();
  await expect(page).toHaveURL(/\/flows\/flw_promos$/);
});
