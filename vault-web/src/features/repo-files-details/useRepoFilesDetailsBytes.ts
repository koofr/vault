import { useMemo } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';

export function useRepoFilesDetailsBytes(
  detailsId: number,
): ArrayBufferView<ArrayBufferLike> | undefined {
  const [arrayBuffer] = useSubscribe(
    (v, cb) => v.repoFilesDetailsContentBytesSubscribe(detailsId, cb),
    (v) => v.repoFilesDetailsContentBytesData,
    [detailsId],
  );

  return arrayBuffer;
}

export function useRepoFilesDetailsString(
  detailsId: number,
): string | undefined {
  const arrayBuffer = useRepoFilesDetailsBytes(detailsId);

  const dataString = useMemo(() => {
    return arrayBuffer !== undefined
      ? new TextDecoder('utf-8').decode(arrayBuffer)
      : undefined;
  }, [arrayBuffer]);

  return dataString;
}
