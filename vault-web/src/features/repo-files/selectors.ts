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

export const fileHasDetails = (file: RepoFile): boolean => false;
