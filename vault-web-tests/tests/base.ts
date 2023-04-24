import { test as base } from '@playwright/test';

import { storageState } from '../storageState';
import { config, ignoreHTTPSErrors } from '../vaultConfig';

import { KoofrApiClient } from './koofrApiClient';

interface OAuth2Token {
  access_token: string;
  refresh_token: string;
}

export const test = base.extend<{
  koofrApiClient: KoofrApiClient;
}>({
  koofrApiClient: [
    async ({ request, baseURL }, use) => {
      const oauth2Token: OAuth2Token = JSON.parse(
        storageState.origins[0].localStorage[0].value
      );

      const apiBaseUrl = config.baseUrl;
      const oauth2ClientId = config.oauth2ClientId;
      const oauth2ClientSecret = config.oauth2ClientSecret;
      const oauth2RedirectUri = `${baseURL}/oauth2callback`;

      const koofrApiClient = new KoofrApiClient(
        request,
        apiBaseUrl,
        oauth2Token,
        oauth2ClientId,
        oauth2ClientSecret,
        oauth2RedirectUri,
        ignoreHTTPSErrors
      );

      await koofrApiClient.refreshToken();

      await koofrApiClient.removeAllVaultRepos();

      await use(koofrApiClient);
    },
    { auto: true },
  ],
});
