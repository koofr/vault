import { To } from 'react-router-dom';

import { FileCategory, RepoFile } from '../../vault-wasm/vault-wasm';

export const repoFilesLink = (
  repoId: string,
  path?: string,
  name?: string,
): To => {
  const search = new URLSearchParams();

  if (path !== undefined) {
    search.set('path', path);
  }

  if (name !== undefined) {
    search.set('name', name);
  }

  return {
    pathname: `/repos/${repoId}`,
    search: search.toString(),
  };
};

export const repoFilesDetailsLink = (
  repoId: string,
  path: string,
  isEditing?: boolean,
  autosaveIntervalMs?: number,
): To => {
  const search = new URLSearchParams({
    path,
  });

  if (isEditing) {
    search.set('editing', 'true');
  }

  if (autosaveIntervalMs !== undefined) {
    search.set('autosave', `${autosaveIntervalMs}`);
  }

  return {
    pathname: `/repos/${repoId}/details`,
    search: search.toString(),
  };
};

export const fileHasPdfViewer = (ext: string | undefined): boolean =>
  ext === 'pdf';

export const fileHasTextEditor = (
  category: FileCategory | undefined,
): boolean => category === 'Text' || category === 'Code';

export const fileHasImageViewer = (ext: string | undefined): boolean =>
  ext === 'jpg' ||
  ext === 'jpeg' ||
  ext === 'gif' ||
  ext === 'png' ||
  ext === 'svg';

export const fileHasDetails = (file: RepoFile): boolean =>
  !file.nameError &&
  (fileHasPdfViewer(file.ext) ||
    fileHasTextEditor(file.category) ||
    fileHasImageViewer(file.ext));

export const fileCategoryHasDetailsEdit = (
  category: FileCategory | undefined,
): boolean => fileHasTextEditor(category);

export const fileHasDetailsEdit = (file: RepoFile): boolean =>
  !file.nameError && fileCategoryHasDetailsEdit(file.category);
