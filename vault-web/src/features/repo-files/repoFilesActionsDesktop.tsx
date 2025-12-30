import { WebVaultDesktop } from '../../desktopVault/WebVaultDesktop';
import { WebVault } from '../../vault-wasm/vault-wasm';

export const openFileDesktop = (
  webVault: WebVault,
  repoId: string,
  encryptedPath: string,
) => {
  (webVault as WebVaultDesktop).repoFilesOpenFile(repoId, encryptedPath);
};

export const downloadFileDesktop = (
  webVault: WebVault,
  repoId: string,
  encryptedPath: string,
) => {
  (webVault as WebVaultDesktop).repoFilesDownloadFile(repoId, encryptedPath);
};

export const downloadSelectedDesktop = (
  webVault: WebVault,
  browserId: number,
) => {
  (webVault as WebVaultDesktop).repoFilesBrowsersDownloadSelected(browserId);
};
