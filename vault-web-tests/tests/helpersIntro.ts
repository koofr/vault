import { expect, Page } from '@playwright/test';

export async function hideIntro(page: Page) {
  const dialog = page.getByRole('dialog');

  await expect(dialog).toBeVisible();

  await dialog.getByRole('button', { name: 'Close' }).click();
}
