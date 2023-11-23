import { Agent } from 'undici';

import vaultWasm, {
  BrowserHttpClientDelegate,
  Repo,
} from '../vault-wasm-nodejs/vault-wasm.js';
import { BrowserEventstreamWebSocketDelegate } from '../vault-wasm-nodejs/vault-wasm.js';

import { splitParentName } from './pathUtils';
import { sleep } from './time';

const { initConsole, WebVault } = vaultWasm;

let _initConsoleCalled = false;

function tryInitConsole() {
  if (!_initConsoleCalled) {
    initConsole();
    _initConsoleCalled = true;
  }
}

type WebVault = InstanceType<typeof WebVault>;

export class WebVaultClient {
  webVault: WebVault;

  _notificationsUnsubscribe?: () => void;

  constructor(
    baseUrl: string,
    oauth2Token: string,
    oauth2ClientId: string,
    oauth2ClientSecret: string,
    oauth2RedirectUri: string,
    ignoreHTTPSErrors: boolean,
  ) {
    tryInitConsole();

    const storage = new MemoryStorage();

    storage.setItem('vaultOAuth2Token', oauth2Token);

    const browserHttpClientDelegate: BrowserHttpClientDelegate = {
      async fetch(request) {
        if (ignoreHTTPSErrors) {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          (request as any).dispatcher = new Agent({
            connect: {
              rejectUnauthorized: false,
            },
          });
        }

        return await fetch(request.url, request);
      },

      xhr(): Promise<Response> {
        throw new Error('xhr not implemented.');
      },
    };

    const browserEventstreamWebSocketDelegate: BrowserEventstreamWebSocketDelegate =
      {
        open() {
          // not implemented, but should not throw an error
        },
        send() {
          throw new Error('send not implemented.');
        },
        close(): void {
          // not implemented, but should not throw an error
        },
      };

    this.webVault = new WebVault(
      baseUrl,
      baseUrl,
      oauth2ClientId,
      oauth2ClientSecret,
      oauth2RedirectUri,
      browserHttpClientDelegate,
      browserEventstreamWebSocketDelegate,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      storage as any,
    );

    this._subscribeNotifications();
  }

  destroy() {
    this._unsubscribeNotifications();

    this.webVault.free();
  }

  subscribe<T>(
    subscribe: (webVault: WebVault, callback: () => void) => number,
    getDataFunc: (webVault: WebVault) => (subscriptionId: number) => T,
    callback: (data: T, unsubscribe: () => void) => void,
  ): () => void {
    // eslint-disable-next-line prefer-const
    let subscriptionId: number | undefined;

    const getData = getDataFunc(this.webVault);

    const unsubscribe = () => {
      if (subscriptionId !== undefined) {
        this.webVault.unsubscribe(subscriptionId);
        subscriptionId = undefined;
      }
    };

    const subscribeCallback = () => {
      if (subscriptionId !== undefined) {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        callback(getData.call(this.webVault, subscriptionId), unsubscribe);
      }
    };

    subscriptionId = subscribe(this.webVault, () => {
      subscribeCallback();
    });

    subscribeCallback();

    return unsubscribe;
  }

  async load() {
    await this.webVault.load();

    await this.waitForReposLoaded();
  }

  async waitFor<T>(
    subscribe: (webVault: WebVault, callback: () => void) => number,
    getDataFunc: (webVault: WebVault) => (subscriptionId: number) => T,
    check: (data: T) => boolean,
    timeoutMs?: number,
  ): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      let timeoutId: ReturnType<typeof setTimeout> | undefined;

      const unsubscribe = this.subscribe(
        subscribe,
        getDataFunc,
        (data, unsubscribe) => {
          if (check(data)) {
            unsubscribe();

            resolve(data);

            if (timeoutId !== undefined) {
              clearTimeout(timeoutId);
            }
          }
        },
      );

      if (timeoutMs !== undefined) {
        timeoutId = setTimeout(() => {
          unsubscribe();

          reject(new Error(`waitFor timeout after ${timeoutMs} ms`));
        }, timeoutMs);
      }
    });
  }

  // repos

  async waitForReposLoaded() {
    await this.waitFor(
      (v, cb) => v.reposSubscribe(cb),
      (v) => v.reposData,
      (repos) => repos.status.type === 'Loaded',
    );
  }

  async waitForRepo(): Promise<Repo> {
    const repos = await this.waitFor(
      (v, cb) => v.reposSubscribe(cb),
      (v) => v.reposData,
      (repos) => repos.status.type === 'Loaded' && repos.repos.length > 0,
    );

    return repos.repos[0];
  }

  async unlockRepo(repo: Repo, password = 'password'): Promise<void> {
    const unlockId = this.webVault.repoUnlockCreate(repo.id, {
      mode: 'Unlock',
    });

    try {
      await this.webVault.repoUnlockUnlock(unlockId, password);
    } finally {
      this.webVault.repoUnlockDestroy(unlockId);
    }
  }

  // repo files

  async getFileContent(
    repo: Repo,
    encryptedPath: string,
    timeoutMs?: number,
  ): Promise<string> {
    const detailsId = this.webVault.repoFilesDetailsCreate(
      repo.id,
      encryptedPath,
      false,
      {
        autosaveIntervalMs: 20000,
        loadContent: {
          categories: [],
          exts: [],
        },
      },
    );

    try {
      await this.waitFor(
        (v, cb) => v.repoFilesDetailsInfoSubscribe(detailsId, cb),
        (v) => v.repoFilesDetailsInfoData,
        (info) =>
          info?.status.type === 'Loaded' || info?.status.type === 'Error',
        timeoutMs,
      );

      const stream = await this.webVault.repoFilesDetailsGetFileStream(
        detailsId,
        false,
        new AbortController().signal,
      );

      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      return await new Response(stream!.stream!).text();
    } finally {
      await this.webVault.repoFilesDetailsDestroy(detailsId);
    }
  }

  async waitForFileContent(
    repo: Repo,
    encryptedPath: string,
    expectedContent: string,
    timeoutMs: number,
    sleepMs = 25,
  ): Promise<void> {
    const deadline = Date.now() + timeoutMs;

    let lastErr: unknown;

    while (Date.now() < deadline) {
      try {
        const content = await this.getFileContent(
          repo,
          encryptedPath,
          timeoutMs,
        );

        if (content === expectedContent) {
          return;
        }
      } catch (e) {
        lastErr = e;
      }

      await sleep(sleepMs);
    }

    throw new Error(
      `waitForFileContent timeout in ${timeoutMs} ms: ${lastErr}`,
    );
  }

  async setFileContent(
    repo: Repo,
    encryptedPath: string,
    ext: string,
    content: string,
    timeoutMs?: number,
  ) {
    const detailsId = this.webVault.repoFilesDetailsCreate(
      repo.id,
      encryptedPath,
      true,
      {
        autosaveIntervalMs: 20000,
        loadContent: {
          categories: [],
          exts: [ext],
        },
      },
    );

    try {
      await this.waitFor(
        (v, cb) => v.repoFilesDetailsInfoSubscribe(detailsId, cb),
        (v) => v.repoFilesDetailsInfoData,
        (info) => info?.contentStatus.type === 'Loaded',
        timeoutMs,
      );

      this.webVault.repoFilesDetailsSetContent(
        detailsId,
        new TextEncoder().encode(content),
      );

      await this.webVault.repoFilesDetailsSave(detailsId);

      await this.waitFor(
        (v, cb) => v.repoFilesDetailsInfoSubscribe(detailsId, cb),
        (v) => v.repoFilesDetailsInfoData,
        (info) => {
          return info?.isDirty === false;
        },
        timeoutMs,
      );
    } finally {
      await this.webVault.repoFilesDetailsDestroy(detailsId);
    }
  }

  async ensureFile(repo: Repo, encryptedPath: string, timeoutMs?: number) {
    const [encryptedParentPath] = splitParentName(encryptedPath);

    const browserId = this.webVault.repoFilesBrowsersCreate(
      repo.id,
      encryptedParentPath,
      {
        selectName: undefined,
      },
    );

    try {
      await this.waitFor(
        (v, cb) => v.repoFilesBrowsersInfoSubscribe(browserId, cb),
        (v) => v.repoFilesBrowsersInfoData,
        (info) => {
          return info?.status.type === 'Loaded';
        },
        timeoutMs,
      );
    } finally {
      this.webVault.repoFilesBrowsersDestroy(browserId);
    }
  }

  async renameFile(
    repo: Repo,
    encryptedPath: string,
    newName: string,
    timeoutMs?: number,
  ) {
    await this.ensureFile(repo, encryptedPath, timeoutMs);

    const promptDialogFillPromise = this.promptDialogFill(newName);

    await this.webVault.repoFilesRenameFile(repo.id, encryptedPath);

    await promptDialogFillPromise;
  }

  async deleteFile(repo: Repo, encryptedPath: string, timeoutMs?: number) {
    await this.ensureFile(repo, encryptedPath, timeoutMs);

    const confirmDialogPromise = this.confirmDialog();

    await this.webVault.repoFilesDeleteFile(repo.id, encryptedPath);

    await confirmDialogPromise;
  }

  async promptDialogFill(value: string) {
    await this.confirmDialog(value);
  }

  async confirmDialog(inputValue?: string) {
    const dialogId = (
      await this.waitFor(
        (v, cb) => v.dialogsSubscribe(cb),
        (v) => v.dialogsData,
        (dialogs) => dialogs.length > 0,
        5000,
      )
    )[0];

    await this.waitFor(
      (v, cb) => v.dialogsDialogSubscribe(dialogId, cb),
      (v) => v.dialogsDialogData,
      (dialog) => dialog !== undefined,
      5000,
    );

    if (inputValue !== undefined) {
      this.webVault.dialogsSetInputValue(dialogId, inputValue);
    }

    this.webVault.dialogsConfirm(dialogId);

    await this.waitFor(
      (v, cb) => v.dialogsDialogSubscribe(dialogId, cb),
      (v) => v.dialogsDialogData,
      (dialog) => dialog === undefined,
      5000,
    );
  }

  _subscribeNotifications() {
    this._notificationsUnsubscribe = this.subscribe(
      (v, cb) => v.notificationsSubscribe(cb),
      (v) => v.notificationsData,
      (notifications) => {
        if (notifications.length > 0) {
          const notification = notifications[0];
          console.warn(`WebVault notification: ${notification.message}`);
          this.webVault.notificationsRemove(notification.id);
        }
      },
    );
  }

  _unsubscribeNotifications() {
    if (this._notificationsUnsubscribe !== undefined) {
      this._notificationsUnsubscribe();
      this._notificationsUnsubscribe = undefined;
    }
  }
}

export class MemoryStorage {
  data: Map<string, string>;

  constructor() {
    this.data = new Map();
  }

  getItem(key: string): string | null {
    return this.data.get(key) ?? null;
  }

  setItem(key: string, value: string) {
    this.data.set(key, value);
  }

  removeItem(key: string) {
    this.data.delete(key);
  }
}
