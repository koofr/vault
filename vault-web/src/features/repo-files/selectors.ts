import { To } from 'react-router-dom';

import { RepoFile } from '../../vault-wasm/vault-wasm';

export const repoFilesLink = (repoId: string, path: string): To => ({
  pathname: `/repos/${repoId}`,
  search: `path=${encodeURIComponent(path)}`,
});

export const repoFilesDetailsLink = (repoId: string, path: string): To => ({
  pathname: `/repos/${repoId}/details`,
  search: `path=${encodeURIComponent(path)}`,
});

export const fileHasPdfViewer = (file: RepoFile): boolean =>
  file.iconType === 'Pdf' && !file.nameError;

export const fileHasDetails = (file: RepoFile): boolean =>
  fileHasPdfViewer(file);

export const pdfViewerUrl = (fileUrl: string): string =>
  '/pdfjs-3.5.141/web/viewer.html?file=' + encodeURIComponent(fileUrl);
