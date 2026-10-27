import { expect, test } from '@playwright/test';

// S4 Publish/Connect, in fixture mode.

test('publishes an AI app and reveals a key once', async ({ page }) => {
  await page.goto('/flows/flw_larkspur/publish');
  await expect(page.getByRole('heading', { name: /Publish Larkspur Support/ })).toBeVisible();
  await page.getByRole('button', { name: 'Publish' }).click();
  await expect(page.getByRole('heading', { name: /Larkspur Support is live/ })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Open playground' })).toHaveAttribute('href', '/flows/flw_larkspur/chat');

  await page.getByRole('button', { name: 'Create a key' }).click();
  await expect(page.getByText('il_live_5Hq8Lw2Zp9Vt3f9aK1mX')).toBeVisible();
  await expect(page.getByText('This key is shown once.')).toBeVisible();

  await page.getByRole('tab', { name: 'Python' }).click();
  await expect(page.getByText('from openai import OpenAI')).toBeVisible();
  await expect(page.getByLabel('INTERLOCK.md preview')).toContainText('Larkspur Support · integration guide');
});

test('turns an automation off and on', async ({ page }) => {
  await page.goto('/flows/flw_promos/publish');
  await expect(page.getByRole('heading', { name: /Weekly product promos is live/ })).toBeVisible();
  await page.getByRole('button', { name: 'Turn off' }).click();
  await expect(page.getByRole('button', { name: 'Activate' })).toBeVisible();
  await page.getByRole('button', { name: 'Activate' }).click();
  await expect(page.getByRole('heading', { name: /is live/ })).toBeVisible();
});
