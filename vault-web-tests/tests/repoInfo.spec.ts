import { expect } from '@playwright/test';
import { openApp } from '../helpers/app';
import { test } from '../helpers/base';
import { createRepo } from '../helpers/repos';

test.describe('repoInfo', () => {
  test('set auto lock after', async ({ page, debugClient }) => {
    await createRepo(debugClient);

    await openApp(page);

    await page.getByLabel('Safe Box info').click();
    await page
      .getByLabel('Lock Safe Box after')
      .selectOption('Inactive10Minutes');

    await page.reload();

    await expect(page.getByLabel('Lock Safe Box after')).toHaveValue(
      'Inactive10Minutes',
    );
  });

  test('set auto lock on app hidden', async ({ page, debugClient }) => {
    await createRepo(debugClient);

    await openApp(page);

    await page.getByLabel('Safe Box info').click();

    await page.getByLabel('Lock Safe Box on app hidden').click();

    await page.reload();

    await expect(page.getByLabel('Lock Safe Box on app hidden')).toBeChecked();
  });
});
