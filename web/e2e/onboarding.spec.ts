import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

// Q35: the first-run welcome, the guided tour over the real screens, and the Help dialog.
test.use({ storageState: { cookies: [], origins: [] } });

async function noSeriousA11yIssues(page: Page) {
  const { violations } = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
  expect(violations.filter((v) => v.impact === 'serious' || v.impact === 'critical').map((v) => v.id)).toEqual([]);
}

// Each tour step: its title and the screen it is shown on (the fixture workspace's AI app is flw_larkspur).
const steps: [string, RegExp][] = [
  ['Say what it should do', /\/new$/],
  ['Give it what it should know', /\/new$/],
  ['Apps and APIs', /\/new$/],
  ['Build it and watch', /\/new$/],
  ['Two kinds of flow', /\/flows$/],
  ['The canvas', /\/flows\/flw_larkspur$/],
  ['The step panel', /\/flows\/flw_larkspur$/],
  ['Add step', /\/flows\/flw_larkspur$/],
  ['Refine in plain words', /\/flows\/flw_larkspur$/],
  ['Test run', /\/flows\/flw_larkspur$/],
  ['The Playground', /\/flows\/flw_larkspur\/chat$/],
  ['Approvals', /\/approvals$/],
  ['Connections', /\/connections$/],
  ['Publish', /\/flows\/flw_larkspur\/publish$/],
  ['That’s the tour', /\/flows$/]
];

test('a first visit is welcomed, toured over every screen, and remembered', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/flows');
  const welcome = page.getByRole('dialog', { name: 'Welcome to Interlock' });
  await expect(welcome.getByText('Automation', { exact: true })).toBeVisible();
  await expect(welcome.getByText('AI app', { exact: true })).toBeVisible();
  await noSeriousA11yIssues(page);
  await welcome.getByRole('button', { name: 'Take the tour' }).click();

  for (const [i, [title, url]] of steps.entries()) {
    const card = page.getByRole('dialog', { name: title });
    await expect(card.getByText(`${i + 1} of ${steps.length}`)).toBeVisible();
    await expect(page).toHaveURL(url);
    // Focus sits on Next, so Enter moves on.
    await expect(card.getByRole('button', { name: i === steps.length - 1 ? 'Done' : 'Next' })).toBeFocused();
    if (i === 0) await noSeriousA11yIssues(page);
    if (i === 1) {
      await page.keyboard.press('ArrowLeft');
      await expect(page.getByRole('dialog', { name: steps[0]![0] })).toBeVisible();
      await page.keyboard.press('ArrowRight');
    }
    await page.keyboard.press('Enter');
  }
  await expect(page.locator('dialog[open]')).toHaveCount(0);

  await page.reload();
  await expect(page.getByRole('heading', { name: 'Flows', level: 1 })).toBeVisible();
  await expect(page.locator('dialog[open]')).toHaveCount(0);
});

test('Escape ends the tour, and Help brings back the checklist and the welcome', async ({ page }) => {
  await page.goto('/flows');
  await page.getByRole('dialog', { name: 'Welcome to Interlock' }).getByRole('button', { name: 'Skip for now' }).click();

  await page.getByRole('button', { name: 'Help' }).click();
  const help = page.getByRole('dialog', { name: 'Getting started' });
  await expect(help.getByText('4 of 4 done')).toBeVisible();
  await expect(help.getByRole('link', { name: /Connect an app/ })).toHaveAttribute('href', '/connections');

  await help.getByRole('button', { name: 'Take the tour' }).click();
  await expect(page.getByRole('dialog', { name: 'Say what it should do' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.locator('dialog[open]')).toHaveCount(0);

  await page.getByRole('button', { name: 'Help' }).click();
  await page.getByRole('dialog', { name: 'Getting started' }).getByRole('button', { name: 'Show the welcome' }).click();
  await expect(page.getByRole('dialog', { name: 'Welcome to Interlock' })).toBeVisible();
});

test('on a phone, Settings has the tour and the checklist', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/settings');
  await page.getByRole('dialog', { name: 'Welcome to Interlock' }).getByRole('button', { name: 'Skip for now' }).click();
  const section = page.getByRole('region', { name: 'Getting started' }).first();
  await expect(section.getByText('4 of 4 done')).toBeVisible();
  await section.getByRole('button', { name: 'Take the tour' }).click();
  await expect(page.getByRole('dialog', { name: 'Say what it should do' })).toBeVisible();
  await expect(page).toHaveURL(/\/new$/);
});
