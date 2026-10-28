import { expect, test } from '@playwright/test';

// Node panel in fixture mode: schema form, field mapper, Run this step, credential picker.
const URL = '/flows/flw_promos?speed=60';

test('edits a step, maps a field and runs the step', async ({ page }) => {
  await page.goto(URL);
  await page.getByTestId('node-n_script').click();
  const panel = page.getByTestId('node-panel');
  await expect(panel.getByRole('heading', { name: 'Write video script' })).toBeVisible();
  await expect(panel.getByLabel('Tone')).toHaveValue('upbeat');

  const instructions = panel.getByLabel('Instructions *');
  await instructions.fill('Write a short promo for ');
  const field = panel.locator('.field', { has: page.getByLabel('Instructions *') });
  await field.getByRole('button', { name: 'Insert field' }).click();
  await page.getByRole('listbox', { name: 'Fields from earlier steps' }).getByRole('option').first().click();
  await expect(instructions).toHaveValue('Write a short promo for {{ item }}');
  await expect(field.getByLabel('Mapped fields')).toContainText('Item');

  await instructions.fill('');
  await expect(panel.getByText('Instructions is required')).toBeVisible();
  await instructions.fill('Write a 20-second promo.');
  await expect(panel.getByRole('button', { name: 'Run this step' })).toBeDisabled();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await panel.getByRole('button', { name: 'Run this step' }).click();
  await expect(panel.getByRole('tab', { name: 'Output' })).toHaveAttribute('aria-selected', 'true');
  await expect(panel.getByText('Meet the Larkspur Linen Tote')).toBeVisible({ timeout: 15_000 });
});

test('picks a connected account or offers to connect', async ({ page }) => {
  await page.goto(URL);
  await page.getByRole('button', { name: 'List view' }).click();
  await page.getByRole('button', { name: /^Approve in Slack,/ }).click();
  const panel = page.getByTestId('node-panel');
  await panel.getByLabel('Slack workspace (optional)').selectOption({ label: 'Larkspur workspace' });
  await expect(page.getByText('Unsaved changes')).toBeVisible();

  await panel.getByRole('button', { name: 'Close panel' }).click();
  await page.getByRole('button', { name: /^Bluesky,/ }).click();
  await expect(panel.getByTestId('connect-bluesky').getByRole('link', { name: 'Connect' })).toHaveAttribute('href', '/connections');
});

test('a trigger step says how it starts the flow, and an Off flow says how to try it', async ({ page }) => {
  await page.goto('/flows/flw_promos');
  await page.getByTestId('node-n_schedule').dispatchEvent('click');
  await expect(page.getByRole('region', { name: 'How this step starts the flow' })).toContainText('Every Monday at 09:00');

  await page.goto('/flows/flw_larkspur');
  await expect(page.getByRole('note').filter({ hasText: 'This flow is off.' })).toBeVisible();
});
