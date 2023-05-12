import { expect } from '@playwright/test';

import { test } from '../helpers/base';
import { openApp } from '../helpers/app';
import { unlockRepo } from '../helpers/repos';
import { hideIntro } from '../helpers/intro';
import { filesTableRowNameLocator } from '../helpers/repoFiles';

test('create a new safe box', async ({ page }) => {
  await openApp(page);
  await hideIntro(page);

  // firefox needs click before fill
  await page.getByLabel('Safe Key').click();
  await page.getByLabel('Safe Key').fill('password');
  await page.getByRole('button', { name: 'Show advanced settings' }).click();
  // firefox needs click before fill
  await page.getByLabel('Salt').click();
  await page.getByLabel('Salt').fill('salt');
  await page.getByRole('button', { name: 'Create' }).click();
  await page.getByRole('button', { name: 'Copy' }).click();
  await expect(page.getByRole('button', { name: 'Copied' })).toBeVisible();
  await page.getByRole('button', { name: 'Continue' }).click();

  await unlockRepo(page);

  await expect(
    filesTableRowNameLocator(page, 'Dir', 'My private documents')
  ).toBeVisible();
});
