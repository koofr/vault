import { useCallback, useEffect, useRef, useState } from 'react';

import { useWebVault } from '../../../webVault/useWebVault';
import { useSubscribe } from '../../../webVault/useSubscribe';

export function useRepoFilesDetailsBlobUrl(
  detailsId: number
): string | undefined {
  const webVault = useWebVault();

  const [blobUrl, setBlobUrl] = useState<string>();

  const lastBlobUrl = useRef<string>();
  const revokeLastBlobUrl = useCallback(() => {
    if (lastBlobUrl.current !== undefined) {
      URL.revokeObjectURL(lastBlobUrl.current);
      lastBlobUrl.current = undefined;
    }
  }, []);
  useEffect(() => {
    return revokeLastBlobUrl;
  }, [revokeLastBlobUrl]);

  const loadFile = useCallback(async () => {
    const stream = await webVault.repoFilesDetailsGetFileStream(
      detailsId,
      true
    );

    const blobUrl =
      stream !== undefined && stream.blob !== undefined
        ? URL.createObjectURL(stream.blob)
        : undefined;

    revokeLastBlobUrl();

    lastBlobUrl.current = blobUrl;
    setBlobUrl(blobUrl);
  }, [webVault, detailsId, revokeLastBlobUrl]);

  useSubscribe(
    (v, cb) => v.repoFilesDetailsFileSubscribe(detailsId, cb),
    (v) => (subscriptionId: number) => {
      // if file has changed, just load the file url, no need to call
      // repoFilesDetailsFileData()
      loadFile();
    },
    [detailsId, loadFile]
  );

  return blobUrl;
}
