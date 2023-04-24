import { expect } from '@playwright/test';

import { test } from './base';

test('intro', async ({ page }) => {
  await page.goto('/');

  const dialog = page.getByRole('dialog');

  await expect(dialog).toBeVisible();
  await expect(dialog.getByText('Welcome', { exact: true })).toBeVisible();
  await dialog.getByRole('button', { name: 'Next' }).click();
  await expect(dialog.getByText('About Vault', { exact: true })).toBeVisible();
  await dialog.getByRole('button', { name: 'Next' }).click();
  await expect(dialog.getByText('Safe Boxes', { exact: true })).toBeVisible();
  await dialog.getByRole('button', { name: 'Next' }).click();
  await expect(dialog.getByText('Safe Key', { exact: true })).toBeVisible();
  await dialog.getByRole('button', { name: 'Next' }).click();
  await expect(
    dialog.getByText('Start using Vault', { exact: true })
  ).toBeVisible();
  await dialog.getByRole('button', { name: 'Done' }).click();
  await expect(dialog).not.toBeVisible();
});
