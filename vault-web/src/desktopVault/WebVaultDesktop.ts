import { WebVault } from '../vault-wasm/vault-wasm';
import { WebVaultClient } from './WebVaultClient';

export interface WebVaultDesktop extends WebVault {
  client: WebVaultClient;

  login(): void;
  oauth2Logout(): void;

  repoFilesOpenFile(repoId: string, encryptedPath: string): void;
  repoFilesDownloadFile(repoId: string, encryptedPath: string): void;
  repoFilesUploadFile(repoId: string, encryptedPath: string): void;
  repoFilesUploadDir(repoId: string, encryptedPath: string): void;
  repoFilesUploadPaths(
    repoId: string,
    encryptedPath: string,
    localPaths: string[],
  ): void;

  repoFilesBrowsersDownloadSelected(browserId: number): void;
}
