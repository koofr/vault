import { WebVault } from '../../vault-wasm/vault-wasm';

import {
  downloadFileDesktop,
  downloadSelectedDesktop,
  openFileDesktop,
} from './repoFilesActionsDesktop';
import { downloadFileWeb, downloadSelectedWeb } from './repoFilesActionsWeb';

export const openFile = async (
  webVault: WebVault,
  repoId: string,
  encryptedPath: string,
  isMobile: boolean,
) => {
  if (import.meta.env.VITE_VAULT_APP === 'desktop') {
    openFileDesktop(webVault, repoId, encryptedPath);
  } else {
    downloadFileWeb(webVault, repoId, encryptedPath, isMobile);
  }
};

export const downloadFile = async (
  webVault: WebVault,
  repoId: string,
  encryptedPath: string,
  isMobile: boolean,
) => {
  if (import.meta.env.VITE_VAULT_APP === 'desktop') {
    downloadFileDesktop(webVault, repoId, encryptedPath);
  } else {
    downloadFileWeb(webVault, repoId, encryptedPath, isMobile);
  }
};

export const downloadSelected = async (
  webVault: WebVault,
  browserId: number,
  isMobile: boolean,
) => {
  if (import.meta.env.VITE_VAULT_APP === 'desktop') {
    downloadSelectedDesktop(webVault, browserId);
  } else {
    downloadSelectedWeb(webVault, browserId, isMobile);
  }
};
