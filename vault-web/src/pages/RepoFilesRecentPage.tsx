import { memo, useMemo } from 'react';
import { useParams } from 'react-router-dom';

import { RepoFiles } from '../features/repo-files/RepoFiles';
import { RepoFilesBrowserSource } from '../vault-wasm/vault-wasm';

export const RepoFilesRecentPage = memo(() => {
  const params = useParams();
  const repoId = params.repoId;

  const source = useMemo(
    (): RepoFilesBrowserSource | undefined =>
      repoId !== undefined
        ? {
            type: 'Recent',
            repoId,
          }
        : undefined,
    [repoId],
  );

  if (source === undefined) {
    return null;
  }

  return <RepoFiles source={source} selectName={undefined} />;
});
RepoFilesRecentPage.displayName = 'RepoFilesRecentPage';
