import { createContext, useContext } from 'react';

import { RepoFile } from '../../vault-wasm/vault-wasm';

export const RepoFilesRenameContext = createContext<(file: RepoFile) => void>(
  undefined as any
);

export function useRepoFilesRename() {
  return useContext(RepoFilesRenameContext);
}
