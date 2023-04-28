import { useEffect, useRef, useState } from 'react';

import { RepoFile } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

export function useRepoFilesDetailsBlobUrl(
  detailsId: number,
  file: RepoFile
): string | undefined {
  const webVault = useWebVault();
  const fileId = file.id;
  const [blobUrl, setBlobUrl] = useState<string>();
  const lastBlobUrl = useRef<string>();
  useEffect(() => {
    const getFile = async () => {
      const stream = await webVault.repoFilesDetailsGetFileStream(
        detailsId,
        true
      );

      if (stream === undefined || stream.blob === undefined) {
        return;
      }

      var blobUrl = URL.createObjectURL(stream.blob);

      lastBlobUrl.current = blobUrl;
      setBlobUrl(blobUrl);
    };

    if (fileId !== undefined) {
      getFile();
    }
  }, [webVault, detailsId, fileId]);
  useEffect(() => {
    return () => {
      if (lastBlobUrl.current !== undefined) {
        URL.revokeObjectURL(lastBlobUrl.current);
        lastBlobUrl.current = undefined;
      }
    };
  }, [webVault]);

  return blobUrl;
}
