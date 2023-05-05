import { useEffect } from 'react';

import { RepoFile } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

export function useRepoFilesDetailsBytes(
  detailsId: number,
  file: RepoFile
): ArrayBuffer | undefined {
  const webVault = useWebVault();
  useEffect(() => {
    webVault.repoFilesDetailsLoadContent(detailsId);
  }, [webVault, detailsId, file.repoId, file.path, file.modified]);
  const [arrayBuffer] = useSubscribe(
    (v, cb) => v.repoFilesDetailsContentBytesSubscribe(detailsId, cb),
    (v) => v.repoFilesDetailsContentBytesData,
    [detailsId]
  );

  return arrayBuffer;
}

export function useRepoFilesDetailsString(
  detailsId: number,
  file: RepoFile
): string | undefined {
  const arrayBuffer = useRepoFilesDetailsBytes(detailsId, file);

  return arrayBuffer !== undefined
    ? new TextDecoder('utf-8').decode(arrayBuffer)
    : undefined;
}
