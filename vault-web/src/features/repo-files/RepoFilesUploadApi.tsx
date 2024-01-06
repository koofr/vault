import { createContext, useContext } from 'react';

export interface RepoFilesUploadApi {
  uploadFile?: () => void;
  uploadDir?: () => void;
}

export const RepoFilesUploadApiContext = createContext<RepoFilesUploadApi>(
  undefined as any,
);

export function useRepoFilesUploadApi() {
  const api = useContext(RepoFilesUploadApiContext);

  return api;
}
