import { expect, test } from '@playwright/test';

// Q34: sign-in with the configured providers, a failed sign-in's reason, and deleting an account.

test('sign-in offers each configured provider and shows why a sign-in failed', async ({ page }) => {
  await page.goto('/signin?error=Google%20sign-in%20was%20cancelled.');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Sign in to Interlock');
  await expect(page.getByRole('complementary', { name: 'Main' })).toHaveCount(0);
  await expect(page.getByRole('alert')).toHaveText('Google sign-in was cancelled.');
  await expect(page.getByRole('link', { name: 'Continue with Google' })).toHaveAttribute('href', '/api/auth/google/start');
  await expect(page.getByRole('link', { name: 'Continue with GitHub' })).toHaveAttribute('href', '/api/auth/github/start');
});

test('deleting an account needs DELETE typed out, then leaves for the landing page', async ({ page }) => {
  await page.goto('/profile');
  await page.getByRole('button', { name: 'Delete my account' }).click();
  const dialog = page.getByRole('dialog', { name: 'Delete your account?' });
  const confirm = dialog.getByRole('button', { name: 'Delete everything' });
  await expect(confirm).toBeDisabled();
  await dialog.getByLabel('Type DELETE').fill('DELETE');
  await confirm.click();
  await expect(page).toHaveURL(/\/$/);
});

test('the account menu opens Profile, Settings and Sign out, and the profile shows who is signed in', async ({ page }) => {
  await page.goto('/flows');
  await page.getByRole('button', { name: 'Account: Dana Ruiz' }).click();
  const menu = page.getByRole('menu');
  await expect(menu.getByRole('menuitem')).toHaveText(['Profile', 'Settings', 'Sign out']);
  await menu.getByRole('menuitem', { name: 'Profile' }).click();
  await expect(page).toHaveURL(/\/profile$/);
  await expect(page.getByRole('heading', { name: 'Dana Ruiz' })).toBeVisible();
  await expect(page.getByText('dana@larkspur.example')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
});
