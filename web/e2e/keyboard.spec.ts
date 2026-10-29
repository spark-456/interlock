import { expect, test, type Page } from '@playwright/test';

// Q06: create → build → editor → publish with the keyboard only.
async function tabTo(page: Page, name: RegExp, max = 60) {
  for (let i = 0; i < max; i++) {
    await page.keyboard.press('Tab');
    const label = await page.evaluate(() => {
      const el = document.activeElement as HTMLElement | null;
      return el ? `${el.getAttribute('aria-label') ?? ''} ${el.innerText ?? ''}`.trim() : '';
    });
    if (name.test(label)) return;
  }
  throw new Error(`Tab never reached ${name}`);
}

test('builds and publishes a flow with the keyboard only', async ({ page }) => {
  await page.goto('/new');
  await page.getByLabel('Goal', { exact: true }).focus();
  await page.keyboard.type('Reply to emails sent to support@ using our help center, and log each one in a sheet.');
  await tabTo(page, /^Build it$/);
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/building/);

  await page.goto('/new/bld_fa/building?speed=60');
  await expect(page.getByRole('link', { name: 'Open editor' }).first()).toBeVisible({ timeout: 30_000 });
  await tabTo(page, /Open editor/);
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/flows\/[^/]+$/);

  await tabTo(page, /^(Activate|Connect)$/);
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/publish$/);
});
