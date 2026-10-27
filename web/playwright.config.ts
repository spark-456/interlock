import { defineConfig, devices } from '@playwright/test';

// Smoke tests run against the fixture-mode dev server: no backend, no keys.
export default defineConfig({
  testDir: 'e2e',
  timeout: 60_000,
  use: { baseURL: 'http://127.0.0.1:4173' },
  webServer: {
    command: 'pnpm exec cross-env VITE_FIXTURES=1 VITE_FIXTURE_SPEED=20 vite dev --host 127.0.0.1 --port 4173 --strictPort',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }]
});
