import { test as setup } from '@playwright/test';

import { refreshOAuth2Tokens } from '../helpers/storageState';

setup('refresh oauth2 token', async () => {
  await refreshOAuth2Tokens(true);
});
