import { expect, test } from '@playwright/test';

// The /dev/components gallery: interactive components work with keyboard and pointer.

test('menu, checkbox, tooltip and drop zone behave', async ({ page }) => {
  await page.goto('/dev/components');
  await page.getByRole('button', { name: 'More actions' }).click();
  const menu = page.getByRole('menu');
  await expect(menu.getByRole('menuitem')).toHaveCount(3);
  await page.keyboard.press('ArrowDown');
  await expect(menu.getByRole('menuitem', { name: 'Rename' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByText('Picked: Rename')).toBeVisible();
  await expect(page.getByRole('menu')).toHaveCount(0);

  const box = page.getByRole('checkbox', { name: 'Include sources' });
  await expect(box).toBeChecked();
  await page.getByText('Include sources').click();
  await expect(box).not.toBeChecked();

  await expect(page.getByRole('tooltip', { name: 'Copies the key' })).toBeAttached();
  await page.getByTestId('file-input').setInputFiles({ name: 'notes.md', mimeType: 'text/markdown', buffer: Buffer.from('# Notes') });
  await expect(page.getByText('Got: notes.md')).toBeVisible();
});
