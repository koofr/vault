import { memo, useMemo } from 'react';
import { useParams, useSearchParams } from 'react-router-dom';

import { RepoFiles } from '../features/repo-files/RepoFiles';
import { useSelectName } from '../features/repo-files/useSelectName';
import { RepoFilesBrowserSource } from '../vault-wasm/vault-wasm';

export const RepoFilesPage = memo(() => {
  const params = useParams();
  const repoId = params.repoId;

  const [searchParams] = useSearchParams();
  const path = searchParams.get('path') ?? undefined;

  const source = useMemo(
    (): RepoFilesBrowserSource | undefined =>
      repoId !== undefined
        ? {
            type: 'Storage',
            repoId,
            encryptedPath: path ?? '/',
          }
        : undefined,
    [repoId, path],
  );

  const selectName = useSelectName(repoId, path);

  if (source === undefined) {
    return null;
  }

  return <RepoFiles source={source} selectName={selectName} />;
});
