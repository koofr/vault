import { useState } from 'react';

import { useWebVaultDesktop } from '../../desktopVault/useWebVaultDesktop';
import { useSubscribe } from '../../webVault/useSubscribe';

export function useRepoFilesDetailsFileDesktopUrl(
  detailsId: number,
): string | undefined {
  const webVault = useWebVaultDesktop();

  const [fileUrl, setFileUrl] = useState<string>();

  useSubscribe(
    (v, cb) => v.repoFilesDetailsFileSubscribe(detailsId, cb),
    (v) => (id) => {
      const file = v.repoFilesDetailsFileData(id);

      if (file !== undefined) {
        // load file on change if file exists
        setFileUrl(
          webVault.client.getUrl('repoFilesDetailsGetFileStream', {
            detailsId: detailsId,
            hash: file.remoteHash,
          }),
        );
      } else {
        setFileUrl(undefined);
      }
    },
    [detailsId],
  );

  return fileUrl;
}
