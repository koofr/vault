import { Locator, Page } from '@playwright/test';

export function filesTableLocator(page: Page): Locator {
  return page.getByLabel('Files list');
}

export function filesTableRowLocator(
  page: Page,
  type: 'Dir' | 'File',
  name: string
): Locator {
  return filesTableLocator(page).getByLabel(
    `${type === 'Dir' ? 'Folder' : 'File'} ${name}`
  );
}

export function filesTableRowNameLocator(
  page: Page,
  type: 'Dir' | 'File',
  name: string
): Locator {
  return filesTableRowLocator(page, type, name)
    .getByRole('cell')
    .nth(1)
    .getByRole('link');
}

export async function uploadFile(page: Page, file: string) {
  await page.getByLabel('Upload file').setInputFiles(file);
}
