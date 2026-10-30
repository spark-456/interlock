import { expect, test } from '@playwright/test';

// S6 editor in fixture mode: add from the palette, undo, Ask diff accept/discard, test run, save.
const URL = '/flows/flw_promos?speed=60';

test('adds a step from the palette and undoes it', async ({ page }) => {
  await page.goto(URL);
  await expect(page.getByTestId('node-n_schedule')).toBeVisible();
  await page.getByRole('button', { name: 'Add step' }).click();
  await page.getByRole('searchbox', { name: 'Search steps' }).fill('wait');
  await page.getByTestId('palette-logic.wait').click();
  await expect(page.getByTestId('node-n_1')).toBeVisible();
  await expect(page.getByText('Unsaved changes')).toBeVisible();

  await page.getByRole('button', { name: 'Undo' }).click();
  await expect(page.getByTestId('node-n_1')).toHaveCount(0);
  await page.getByRole('button', { name: 'Redo' }).click();
  await expect(page.getByTestId('node-n_1')).toBeVisible();
});

test('replaces a step, keeping its place', async ({ page }) => {
  await page.goto(URL);
  await page.getByTestId('node-n_posts').click();
  await page.getByRole('button', { name: 'Replace step' }).click();
  await page.getByTestId('palette-ai.agent').click();
  await expect(page.getByTestId('node-n_posts')).toContainText('Agent');
  await expect(page.getByTestId('node-n_posts')).toContainText('Write post text');
});

test('shows an Ask Interlock diff, then accepts it', async ({ page }) => {
  await page.goto(URL);
  await page.getByRole('button', { name: 'Refine' }).click();
  await page.getByRole('combobox', { name: 'Describe a change' }).fill('Also post to Telegram and drop TikTok');
  await page.getByRole('button', { name: 'Propose' }).click();

  await expect(page.getByTestId('ask-bar')).toContainText('+1 step, 1 changed, 1 removed');
  await expect(page.getByTestId('node-n_telegram')).toHaveAttribute('data-diff', 'added');
  await expect(page.getByTestId('node-n_approve')).toHaveAttribute('data-diff', 'changed');
  await expect(page.getByTestId('node-n_tiktok')).toHaveAttribute('data-diff', 'removed');

  await page.getByRole('button', { name: 'Accept' }).click();
  await expect(page.getByTestId('node-n_tiktok')).toHaveCount(0);
  await expect(page.getByTestId('node-n_telegram')).not.toHaveAttribute('data-diff');
  // Refine stays open for the next change, listing what was kept.
  await expect(page.getByRole('list', { name: 'Changes so far' })).toContainText('Also post to Telegram and drop TikTok (+1 step, 1 changed, 1 removed)');
  await expect(page.getByRole('combobox', { name: 'Describe a change' })).toBeVisible();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Saved')).toBeVisible();
});

test('Q22 Refine is pointed at steps with @ or a click', async ({ page }) => {
  await page.goto(URL);
  await page.getByRole('button', { name: 'Refine' }).click();
  const input = page.getByRole('combobox', { name: 'Describe a change' });
  await input.fill('Make the captions bigger in @cut');
  await page.getByRole('option', { name: 'Cut for each platform' }).waitFor();
  await input.press('Enter');
  const picked = page.getByLabel('Steps to change');
  await expect(picked).toContainText('@Cut for each platform');
  await expect(input).toHaveValue('Make the captions bigger in ');

  await page.getByTestId('node-n_posts').click();
  await expect(picked).toContainText('@Write post text');
  await page.getByRole('button', { name: "Don't change Cut for each platform" }).click();
  await expect(picked).not.toContainText('Cut for each platform');
});

test('a test run animates the canvas with item counts', async ({ page }) => {
  await page.goto(URL);
  await page.getByRole('button', { name: 'Test run' }).click();
  await expect(page.getByTestId('node-n_products')).toContainText('Done', { timeout: 30_000 });
  await expect(page.getByTestId('edge-count').first()).toBeVisible();
  await expect(page.getByText('Test run passed')).toBeVisible({ timeout: 45_000 });
});
