import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

// Q06: no serious or critical axe issues and no sideways scroll on any screen, on desktop and at 390 px.
// Streamed screens are checked once their stream ends, with motion reduced, so nothing is mid-fade.
const done: Record<string, (page: Page) => Promise<void>> = {
  '/new/bld_fa/building?speed=60': (page) =>
    expect(page.getByRole('link', { name: 'Open editor' }).first()).toBeVisible({ timeout: 30_000 }),
  '/flows/flw_promos/runs/run_fa': (page) =>
    expect(page.getByRole('complementary', { name: 'Run details' }).getByText('$0.011')).toBeVisible({ timeout: 30_000 })
};
const screens = [
  '/', '/new', '/new/bld_fa/building?speed=60', '/flows', '/flows/flw_promos', '/flows/flw_promos/runs/run_fa',
  '/flows/flw_promos/publish', '/flows/flw_larkspur/chat', '/templates', '/connections', '/approvals', '/settings'
];

for (const width of [1280, 390]) {
  for (const path of screens) {
    test(`${path} has no serious a11y issues at ${width}px`, async ({ page }) => {
      await page.emulateMedia({ reducedMotion: 'reduce' });
      await page.setViewportSize({ width, height: 844 });
      await page.goto(path);
      await expect(page.locator('main')).toBeVisible();
      await done[path]?.(page);
      const { violations } = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
      const serious = violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
      expect(serious.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).slice(0, 3).join(' | ')}`)).toEqual([]);
      const scroll = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      expect(scroll, 'no sideways page scroll').toBeLessThanOrEqual(0);
    });
  }
}
