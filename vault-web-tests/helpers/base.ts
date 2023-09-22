import { test as base } from '@playwright/test';

import { oauth2RedirectUri } from './baseUrl';
import { DebugClient } from './debugClient';
import { Dialogs } from './dialogs';
import { getInitialOAuth2Token } from './oauth2';
import { config, ignoreHTTPSErrors } from './vaultConfig';
import { WebVaultClient } from './webVaultClient';

export const test = base.extend<{
  debugClient: DebugClient;
  webVaultClient: WebVaultClient;
  dialogs: Dialogs;
}>({
  debugClient: [
    async ({ request }, use) => {
      const debugClient = new DebugClient(request, 'http://127.0.0.1:3080');

      await debugClient.reset();

      await use(debugClient);
    },
    { auto: true },
  ],

  // eslint-disable-next-line no-empty-pattern
  storageState: async ({}, use) => {
    await use({
      cookies: [],
      origins: [
        {
          origin: 'http://localhost:5173',
          localStorage: [
            {
              name: 'vaultOAuth2Token',
              value: getInitialOAuth2Token(),
            },
          ],
        },
      ],
    });
  },

  // eslint-disable-next-line no-empty-pattern
  webVaultClient: async ({}, use) => {
    const apiBaseUrl = config.baseUrl;
    const oauth2Token = getInitialOAuth2Token();
    const oauth2ClientId = config.oauth2ClientId;
    const oauth2ClientSecret = config.oauth2ClientSecret;

    const webVaultClient = new WebVaultClient(
      apiBaseUrl,
      oauth2Token,
      oauth2ClientId,
      oauth2ClientSecret,
      oauth2RedirectUri,
      ignoreHTTPSErrors
    );

    try {
      await use(webVaultClient);
    } finally {
      webVaultClient.destroy();
    }
  },

  dialogs: async ({ page }, use) => {
    await use(new Dialogs(page));
  },
});
