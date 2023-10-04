import { useCallback, useEffect, useRef, useState } from 'react';

import { useWebVault } from '../../../webVault/useWebVault';
import { useSubscribe } from '../../../webVault/useSubscribe';

export function useRepoFilesDetailsBlobUrl(
  detailsId: number,
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

  const lastAbortController = useRef<AbortController>();
  const abortLastAbortController = useCallback(() => {
    if (lastAbortController.current !== undefined) {
      lastAbortController.current.abort();
      lastAbortController.current = undefined;
    }
  }, []);
  useEffect(() => {
    return abortLastAbortController;
  }, [abortLastAbortController]);

  const loadFile = useCallback(async () => {
    abortLastAbortController();

    const abortController = new AbortController();
    lastAbortController.current = abortController;

    const stream = await webVault.repoFilesDetailsGetFileStream(
      detailsId,
      true,
      abortController.signal,
    );

    const blobUrl =
      stream !== undefined && stream.blob !== undefined
        ? URL.createObjectURL(stream.blob)
        : undefined;

    revokeLastBlobUrl();

    lastBlobUrl.current = blobUrl;
    setBlobUrl(blobUrl);
  }, [webVault, detailsId, abortLastAbortController, revokeLastBlobUrl]);

  useSubscribe(
    (v, cb) => v.repoFilesDetailsFileSubscribe(detailsId, cb),
    (v) => (id) => {
      const file = v.repoFilesDetailsFileData(id);

      if (file !== undefined) {
        // load file on change if file exists
        loadFile();
      }
    },
    [detailsId, loadFile],
  );

  return blobUrl;
}
