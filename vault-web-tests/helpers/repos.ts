import { Page, expect } from '@playwright/test';

import { RepoAutoLock, WebVault } from '../vault-wasm-nodejs/vault-wasm';

import { browserWebVaultWait } from './browserWebVault';
import { DebugClient } from './debugClient';

export async function createRepo(client: DebugClient): Promise<string> {
  return await client.createTestVaultRepo();
}

export async function unlockRepoWait(page: Page) {
  await expect(
    page.getByText('Enter your Safe Key to continue', { exact: true }),
  ).toBeVisible();
}

export async function unlockRepo(page: Page) {
  await unlockRepoWait(page);
  // firefox needs click before fill
  await page.getByLabel('Safe Key').click();
  await page.getByLabel('Safe Key').fill('password');
  await page.getByRole('button', { name: 'Continue' }).click();
}

export async function setReposDefaultAutoLock(
  page: Page,
  autoLock: RepoAutoLock,
) {
  await browserWebVaultWait(page);

  await page.evaluate((autoLock) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const webVault = (window as any).webVault as WebVault;
    webVault.reposSetDefaultAutoLock(autoLock);
  }, autoLock);
}

export async function lockRepo(page: Page, repoId: string) {
  await browserWebVaultWait(page);

  await page.evaluate((repoId) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const webVault = (window as any).webVault as WebVault;
    webVault.reposLockRepo(repoId);
  }, repoId);
}
