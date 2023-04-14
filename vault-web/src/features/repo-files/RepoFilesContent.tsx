import { memo } from 'react';

import { ErrorComponent } from '../../components/ErrorComponent';
import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';

import { RepoFilesEmpty } from './RepoFilesEmpty';
import { RepoFilesTable } from './RepoFilesTable';

export const RepoFilesContent = memo<{ info: RepoFilesBrowserInfo }>(
  ({ info }) => {
    return (
      <>
        {info.status.type === 'Error' ? (
          <ErrorComponent error={info.status.error} />
        ) : info.status.type === 'Loaded' && info.totalCount === 0 ? (
          <RepoFilesEmpty />
        ) : (
          <RepoFilesTable info={info} />
        )}
      </>
    );
  }
);
