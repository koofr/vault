import { expect } from '@playwright/test';

import { test } from './base';
import { openApp } from './helpersApp';
import { breadcrumbsLocator } from './helpersDashboard';
import { filesTableRowNameLocator, uploadFile } from './helpersRepoFiles';
import { createUnlockedRepo } from './helpersRepos';
import { viewerIframeLocator } from './helpersReposFilesDetails';

test('open PDF viewer', async ({ page }) => {
  await openApp(page);
  await createUnlockedRepo(page);

  await uploadFile(page, 'test-files/example.pdf');

  await filesTableRowNameLocator(page, 'File', 'example.pdf').click();

  await expect(breadcrumbsLocator(page)).toHaveText('example.pdf');

  await expect(viewerIframeLocator(page)).toBeVisible();

  await expect(
    page.frameLocator('#viewerIframe').locator('id=viewer')
  ).toHaveText(/Example PDF file/, { timeout: 30000 });
});
