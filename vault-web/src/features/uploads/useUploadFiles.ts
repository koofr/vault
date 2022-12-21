import { useCallback } from 'react';

import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from '../repo-files/RepoFilesBrowserId';

import { joinPaths, UploadsHelper } from './UploadsHelper';

export function useUploadFiles(): (
  files: File[] | DataTransferItem[]
) => Promise<void>[] {
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();

  const uploadFiles = useCallback(
    (files: File[] | DataTransferItem[]): Promise<void>[] => {
      // we need to get current repoId and path before calling UploadsHelper
      // because user could change the current directory while UploadsHelper is
      // processing files and files could be uploaded to incorrect location
      let { repoId, path } = webVault.repoFilesBrowsersInfo(browserId)!;

      const helper = new UploadsHelper({
        upload(entries) {
          return entries.map(async (entry) => {
            const fullPath = joinPaths(path, entry.parentPath);

            await webVault.uploadsUpload(
              repoId,
              fullPath,
              entry.name,
              entry.file
            );
          });
        },
      });

      return helper.uploadFiles(files);
    },
    [webVault, browserId]
  );

  return uploadFiles;
}
