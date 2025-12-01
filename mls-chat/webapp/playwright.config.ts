import { defineConfig, devices } from '@playwright/test';

const serverPath = '../server';

export default defineConfig({
  testDir: './e2e',
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
      command: `cd ${serverPath} && cargo run`,
      port: 3000,
      reuseExistingServer: true,
      timeout: 180 * 1000,
    },
    {
      command: 'npm run dev',
      port: 4000,
      reuseExistingServer: true,
      timeout: 30 * 1000,
    },
  ],
});
