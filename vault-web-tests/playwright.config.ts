import { defineConfig, devices } from '@playwright/test';

import { baseUrl } from './helpers/baseUrl';
import { workersCount } from './helpers/storageState';
import { ignoreHTTPSErrors } from './helpers/vaultConfig';

const defaultUse = {
  contextOptions: {
    ignoreHTTPSErrors,
  },
};

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: './tests',
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: workersCount,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: 'html',
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: baseUrl,

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',
  },

  /* Configure projects for major browsers */
  projects: [
    { name: 'setup', testMatch: /.*\.setup\.ts/ },

    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        ...defaultUse,
      },
      dependencies: ['setup'],
    },

    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        ...defaultUse,
      },
      dependencies: ['setup'],
    },

    // {
    //   name: 'webkit',
    //   use: {
    //     ...devices['Desktop Safari'],
    //     ...defaultUse,
    //   },
    //   dependencies: ['setup'],
    // },

    /* Test against mobile viewports. */
    // {
    //   name: 'Mobile Chrome',
    //   use: {
    //     ...devices['Pixel 5'],
    //     ...defaultUse,
    //   },
    //   dependencies: ['setup'],
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: {
    //     ...devices['iPhone 12'],
    //     ...defaultUse,
    //   },
    //   dependencies: ['setup'],
    // },

    /* Test against branded browsers. */
    // {
    //   name: 'Microsoft Edge',
    //   use: {
    //     ...devices['Desktop Edge'],
    //     channel: 'msedge',
    //     ...defaultUse,
    //   },
    //   dependencies: ['setup'],
    // },
    // {
    //   name: 'Google Chrome',
    //   use: {
    //     ...devices['Desktop Chrome'],
    //     channel: 'chrome',
    //     ...defaultUse,
    //   },
    //   dependencies: ['setup'],
    // },
  ],

  /* Run your local dev server before starting the tests */
  webServer: {
    command: process.env.CI ? 'npm run preview' : 'npm run dev',
    cwd: '../vault-web',
    url: baseUrl,
    reuseExistingServer: !process.env.CI,
  },
});
