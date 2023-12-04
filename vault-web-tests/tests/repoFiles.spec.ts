import { openApp } from '../helpers/app';
import { test } from '../helpers/base';
import {
  filesTableFileSelected,
  filesTableSelectFile,
} from '../helpers/repoFiles';
import {
  createRepo,
  lockRepo,
  setReposDefaultAutoLock,
  unlockRepo,
  unlockRepoWait,
} from '../helpers/repos';
import { sleep } from '../helpers/time';

test.describe('repoFiles', () => {
  test.describe('auto lock', () => {
    test('auto lock after', async ({ page, debugClient }) => {
      await createRepo(debugClient);
      await openApp(page);
      await setReposDefaultAutoLock(page, {
        after: { type: 'Custom', seconds: 3 },
        onAppHidden: false,
      });
      await unlockRepo(page);

      for (let i = 0; i < 5; i++) {
        await page.getByText('This folder is empty.').click();

        await sleep(1000);
      }

      await sleep(5000);

      await unlockRepoWait(page);
    });

    test('auto lock on app hidden', async ({ page, debugClient }) => {
      await createRepo(debugClient);
      await openApp(page);
      await setReposDefaultAutoLock(page, {
        after: { type: 'NoLimit' },
        onAppHidden: true,
      });
      await unlockRepo(page);

      // opening a new tab doesn't work:
      // https://github.com/microsoft/playwright/issues/2286
      await page.evaluate(() => {
        Object.defineProperty(document, 'visibilityState', {
          value: 'hidden',
          writable: true,
        });
        document.dispatchEvent(new Event('visibilitychange'));
      });

      await unlockRepoWait(page);
    });

    test('keep selection on lock', async ({
      page,
      debugClient,
      webVaultClient,
    }) => {
      await createRepo(debugClient);

      await webVaultClient.load();
      const repo = await webVaultClient.waitForRepo();
      await webVaultClient.unlockRepo(repo);
      await webVaultClient.uploadFile(
        repo,
        '/',
        'file1.txt',
        new Blob(['test']),
      );
      await webVaultClient.uploadFile(
        repo,
        '/',
        'file2.txt',
        new Blob(['test']),
      );

      await openApp(page);
      await unlockRepo(page);

      await filesTableSelectFile(page, 'File', 'file1.txt');

      await lockRepo(page, repo.id);

      await unlockRepo(page);

      await filesTableFileSelected(page, 'File', 'file1.txt');
    });
  });
});
