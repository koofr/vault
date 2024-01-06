import { memo, useEffect } from 'react';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { useRepoFilesUploadApi } from './RepoFilesUploadApi';
import { useWebVaultDesktop } from '../../desktopVault/useWebVaultDesktop';

export const RepoFilesUploadFormDesktop = memo(() => {
  const webVault = useWebVaultDesktop();
  const uploadApi = useRepoFilesUploadApi();
  const browserId = useRepoFilesBrowserId();

  useEffect(() => {
    uploadApi.uploadFile = () => {
      const { repoId, encryptedPath } =
        webVault.repoFilesBrowsersInfo(browserId)!;

      if (repoId !== undefined && encryptedPath !== undefined) {
        webVault.repoFilesUploadFile(repoId, encryptedPath);
      }
    };
    uploadApi.uploadDir = () => {
      const { repoId, encryptedPath } =
        webVault.repoFilesBrowsersInfo(browserId)!;

      if (repoId !== undefined && encryptedPath !== undefined) {
        webVault.repoFilesUploadDir(repoId, encryptedPath);
      }
    };

    return () => {
      uploadApi.uploadFile = undefined;
      uploadApi.uploadDir = undefined;
    };
  }, [webVault, browserId, uploadApi]);

  return null;
});
