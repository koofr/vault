import { useCallback } from 'react';

import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from '../repo-files/RepoFilesBrowserId';

import { UploadsHelper } from './UploadsHelper';

export function useUploadFiles(): (
  files: File[] | DataTransferItem[],
) => Promise<void>[] {
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();

  const uploadFiles = useCallback(
    (files: File[] | DataTransferItem[]): Promise<void>[] => {
      // we need to get current repoId and path before calling UploadsHelper
      // because user could change the current directory while UploadsHelper is
      // processing files and files could be uploaded to incorrect location
      const { repoId, path } = webVault.repoFilesBrowsersInfo(browserId)!;

      if (repoId === undefined || path === undefined) {
        return [];
      }

      const helper = new UploadsHelper({
        upload(entries) {
          return entries.map(async (entry) => {
            const name =
              entry.parentPath === '/'
                ? entry.name
                : entry.parentPath.slice(1) + '/' + entry.name;

            await webVault.transfersUpload(repoId, path, name, entry.file);
          });
        },
      });

      return helper.uploadFiles(files);
    },
    [webVault, browserId],
  );

  return uploadFiles;
}
