import { expect } from '@playwright/test';

import { test } from './base';
import { openApp } from './helpersApp';
import { createUnlockedRepo } from './helpersRepos';

test('create a new safe box', async ({ page }) => {
  await openApp(page);

  await createUnlockedRepo(page);

  await expect(
    page
      .getByLabel('Files list')
      .getByRole('link', { name: 'My private documents' })
  ).toBeVisible();
});
