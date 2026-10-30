import { expect, test } from '@playwright/test';

// S1 / M1 landing, in fixture mode.

test('the landing pitches, plays the recorded build and leads to your flows', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Your AI backend, built from a sentence.');
  await expect(page.getByRole('complementary', { name: 'Main' })).toHaveCount(0);
  const hero = page.getByRole('figure', { name: /building the Larkspur Support assistant/ });
  await expect(hero.locator('.svelte-flow__node').first()).toBeVisible({ timeout: 15_000 });
  await page.getByRole('tab', { name: 'Python' }).click();
  await expect(page.getByText('from openai import OpenAI')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Get started' }).first()).toHaveAttribute('href', '/flows');
});

test('the landing fits a phone without sideways scrolling', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto('/');
  await expect(page.locator('.svelte-flow__node').first()).toBeVisible({ timeout: 15_000 });
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow).toBeLessThanOrEqual(0);
});

test('Q12 favicon.ico serves an icon instead of a missing page', async ({ request }) => {
  const response = await request.get('/favicon.ico');
  expect(response.status()).toBe(200);
  expect([...((await response.body()).subarray(0, 4))]).toEqual([0, 0, 1, 0]);
});
