import { Locator, Page, expect } from '@playwright/test';

export function filesTableLocator(page: Page): Locator {
  return page.getByLabel('Files list');
}

export function filesTableRowLocator(
  page: Page,
  type: 'Dir' | 'File',
  name: string,
): Locator {
  return filesTableLocator(page).getByLabel(
    `${type === 'Dir' ? 'Folder' : 'File'} ${name}`,
  );
}

export function filesTableRowNameLocator(
  page: Page,
  type: 'Dir' | 'File',
  name: string,
): Locator {
  return filesTableRowLocator(page, type, name)
    .getByRole('cell')
    .nth(1)
    .getByRole('link');
}

export function filesTableRowSizeLocator(
  page: Page,
  type: 'Dir' | 'File',
  name: string,
): Locator {
  return filesTableRowLocator(page, type, name).getByRole('cell').nth(3);
}

export async function filesTableClickFile(
  page: Page,
  type: 'Dir' | 'File',
  name: string,
) {
  await filesTableRowNameLocator(page, type, name).click();
}

export async function filesTableSelectFile(
  page: Page,
  type: 'Dir' | 'File',
  name: string,
) {
  await filesTableRowSizeLocator(page, type, name).click();
}

export async function filesTableFileSelected(
  page: Page,
  type: 'Dir' | 'File',
  name: string,
) {
  await expect(filesTableRowLocator(page, type, name)).toHaveAttribute(
    'aria-selected',
    'true',
  );
}

export async function uploadFile(
  page: Page,
  file:
    | string
    | {
        name: string;
        mimeType: string;
        buffer: Buffer;
      },
) {
  await page.getByLabel('Upload file').setInputFiles(file);
}

export async function uploadTextFile(
  page: Page,
  name: string,
  content: string,
) {
  await uploadFile(page, {
    name: name,
    mimeType: 'text/plain',
    buffer: Buffer.from(content, 'utf-8'),
  });
}

export function fileBrowserUrl(repoId: string, encryptedPath: string) {
  return `/repos/${repoId}?path=${encodeURIComponent(encryptedPath)}`;
}

export async function openFileBrowser(
  page: Page,
  repoId: string,
  encryptedPath: string,
) {
  await page.goto(fileBrowserUrl(repoId, encryptedPath));
}

export async function toolbarEditTextFile(page: Page) {
  await page
    .getByRole('navigation')
    .getByRole('link', { name: 'Edit text' })
    .click();
}
