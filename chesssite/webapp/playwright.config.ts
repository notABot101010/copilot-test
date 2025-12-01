import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:4000',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: [
    {
      command: 'cd /home/runner/work/copilot-test/copilot-test/chesssite/server && DATABASE_URL=sqlite:test_e2e.db?mode=rwc cargo run',
      url: 'http://localhost:4001/api/users',
      reuseExistingServer: true,
      timeout: 120 * 1000,
    },
    {
      command: 'npm run dev',
      url: 'http://localhost:4000',
      reuseExistingServer: true,
      timeout: 30 * 1000,
    },
  ],
});
