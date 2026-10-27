import { expect, test } from '@playwright/test';

// S5 chat playground, in fixture mode: each message replays a recorded test run.

test('asks a question, shows activity, the answer and a source passage', async ({ page }) => {
  await page.goto('/flows/flw_larkspur/chat');
  await page.getByLabel('Message').fill('Can I still get a refund for last month’s invoice?');
  await page.getByRole('button', { name: 'Send' }).click();

  const activity = page.getByLabel('Activity');
  await expect(activity).toContainText('Routed to billing', { timeout: 15_000 });
  await expect(activity).toContainText('lookup_customer');
  await expect(page.getByText(/30-day refund window/)).toBeVisible();

  await page.getByRole('button', { name: /Refund policy 2026\.pdf/ }).click();
  const source = page.getByRole('complementary', { name: 'Source' });
  await expect(source).toContainText('within 30 days of the invoice date');
  await expect(source).toContainText('1 of 3');

  await page.getByRole('button', { name: 'No thanks' }).click();
  await expect(page.locator('.user')).toHaveCount(2);
  await page.getByRole('button', { name: 'New chat' }).click();
  await expect(page.locator('.user')).toHaveCount(0);
});
