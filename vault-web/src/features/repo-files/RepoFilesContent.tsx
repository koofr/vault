import { memo } from 'react';

import { ErrorComponent } from '../../components/ErrorComponent';
import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { RepoFilesEmpty } from './RepoFilesEmpty';
import { RepoFilesTable } from './RepoFilesTable';

export const RepoFilesContent = memo<{
  info: RepoFilesBrowserInfo;
}>(({ info }) => {
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();

  return (
    <>
      {info.status.type === 'Error' && !info.status.loaded ? (
        <ErrorComponent
          error={info.status.error}
          onRetry={() => {
            webVault.repoFilesBrowsersLoadFiles(browserId);
          }}
        />
      ) : info.status.type === 'Loaded' && info.totalCount === 0 ? (
        <RepoFilesEmpty />
      ) : (
        <RepoFilesTable info={info} />
      )}
    </>
  );
});
