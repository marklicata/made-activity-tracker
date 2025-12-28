import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './specs',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  
  use: {
    baseURL: 'http://localhost:1500',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Run dev server before tests
  webServer: {
    command: 'npm run dev -- --config vite.config.ts',
    url: 'http://localhost:1500',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
  
  // Ensure Playwright doesn't pick up vitest globals
  globalSetup: undefined,
  globalTeardown: undefined,
});
