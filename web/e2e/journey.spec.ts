import { expect, test } from '@playwright/test';

// The whole path in fixture mode: landing → demo → create → building → editor → test run → publish → chat.

test('a visitor goes from the landing page to chatting with a published app', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: /Try the demo — no sign-in/ }).click();
  await expect(page).toHaveURL(/\/demo$/);
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page).toHaveURL(/\/new$/);

  await page.getByRole('button', { name: 'Support chat agent' }).click();
  await page.getByRole('button', { name: 'Build it' }).click();
  await expect(page).toHaveURL(/\/new\/bld_larkspur\/building$/);
  await page.getByRole('link', { name: 'Open editor' }).first().click({ timeout: 60_000 });
  await expect(page).toHaveURL(/\/flows\/flw_larkspur$/);

  await page.getByRole('button', { name: 'Test run' }).click();
  await expect(page.getByText(/Test run (passed|failed)/)).toBeVisible({ timeout: 30_000 });

  await page.getByRole('link', { name: 'Connect', exact: true }).click();
  await expect(page).toHaveURL(/\/flows\/flw_larkspur\/publish$/);
  await page.getByRole('button', { name: 'Publish' }).click();
  await page.getByRole('link', { name: 'Open playground' }).click();
  await expect(page).toHaveURL(/\/flows\/flw_larkspur\/chat$/);

  await page.getByLabel('Message').fill('Can I get a refund?');
  await page.getByRole('button', { name: 'Send' }).click();
  await expect(page.getByText(/30-day refund window/)).toBeVisible({ timeout: 15_000 });
});
