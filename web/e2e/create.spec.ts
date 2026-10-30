import { expect, test } from '@playwright/test';

// S2 Create and the Building screen's questions card, in fixture mode.

test('fills a goal from an example, attaches knowledge and starts a build', async ({ page }) => {
  await page.goto('/new');
  const build = page.getByRole('button', { name: 'Build it' });
  await expect(build).toBeDisabled();

  await page.getByRole('button', { name: 'Support inbox replies' }).click();
  await expect(page.getByLabel('Goal', { exact: true })).toHaveValue(/Reply to emails sent to support@/);
  const apps = page.getByRole('group', { name: 'Apps found in your goal' });
  await expect(apps.getByRole('button')).toHaveCount(3);
  for (const name of ['Gmail', 'Google Sheets', 'Slack']) await expect(apps.getByRole('button', { name, exact: true })).toBeVisible();
  await apps.getByRole('button', { name: 'Gmail', exact: true }).click();
  await expect(apps.getByRole('button', { name: 'Gmail', exact: true })).toHaveAttribute('aria-pressed', 'false');

  const knowledge = page.getByRole('region', { name: 'Drop knowledge files' }).getByTestId('file-input');
  await knowledge.setInputFiles({ name: 'Refund policy.pdf', mimeType: 'application/pdf', buffer: Buffer.alloc(2048) });
  const list = page.getByRole('list', { name: 'Attachments' });
  await expect(list).toContainText('Refund policy.pdf');
  await expect(list).toContainText('PDF · 2 KB');
  await list.getByRole('button', { name: 'Instructions' }).click();
  await expect(list.getByRole('button', { name: 'Instructions' })).toHaveAttribute('aria-pressed', 'true');

  await page.getByLabel('Link', { exact: true }).fill('help center');
  await page.getByRole('button', { name: 'Add link' }).click();
  await expect(page.getByText("That doesn't look like a link.", { exact: false })).toBeVisible();
  await page.getByLabel('Link', { exact: true }).fill('https://help.larkspur.example');
  await page.getByRole('button', { name: 'Add link' }).click();
  await expect(list).toContainText('help.larkspur.example');

  // Specs go to APIs, whether dropped in their own box or with the knowledge files.
  const spec = { mimeType: 'application/json', buffer: Buffer.from('{"openapi": "3.0.3"}') };
  await page.getByRole('region', { name: 'Drop an API spec' }).getByTestId('file-input').setInputFiles({ name: 'customers.json', ...spec });
  await knowledge.setInputFiles({ name: 'billing.yaml', ...spec });
  const apisList = page.getByRole('list', { name: 'APIs' });
  await expect(apisList).toContainText('customers.json');
  await expect(apisList).toContainText('billing.yaml');
  await expect(list).not.toContainText('billing.yaml');

  await build.click();
  await expect(page).toHaveURL(/\/new\/bld_larkspur\/building/);
});

test('answers clarifying questions, then shows why a build stopped', async ({ page }) => {
  await page.goto('/new/bld_ask/building?speed=20&at=5');
  const card = page.getByTestId('questions');
  await expect(card).toContainText('Which Slack channel should approvals go to?');
  const go = card.getByRole('button', { name: 'Continue building' });
  await expect(go).toBeDisabled();
  await card.getByRole('button', { name: '#support' }).click();
  await card.getByLabel('Or type your answer').nth(1).fill('The support log');
  await go.click();
  await expect(card).toHaveCount(0);

  await page.goto('/new/bld_ask/building?speed=60');
  await expect(page.getByRole('alert')).toContainText('The flow still had problems after 3 repairs.', { timeout: 15_000 });
  await expect(page.getByRole('link', { name: 'Try again' })).toHaveAttribute('href', '/new');
});
