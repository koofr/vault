import { createContext, useContext } from 'react';

export interface RepoFilesUploadApi {
  uploadFile?: () => void;
  uploadDir?: () => void;
}

export const RepoFilesUploadApiContext = createContext<RepoFilesUploadApi>(
  // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
  undefined as any,
);

export function useRepoFilesUploadApi() {
  const api = useContext(RepoFilesUploadApiContext);

  return api;
}
