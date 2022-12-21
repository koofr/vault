import { memo } from 'react';

import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';

import { RepoFilesError } from './RepoFilesError';
import { RepoFilesTable } from './RepoFilesTable';

export const RepoFilesContent = memo<{ info: RepoFilesBrowserInfo }>(
  ({ info }) => {
    return (
      <>
        {info.status.type === 'Error' ||
        (info.status.type === 'Loaded' && info.totalCount === 0) ? (
          <RepoFilesError info={info} />
        ) : (
          <RepoFilesTable info={info} />
        )}
      </>
    );
  }
);
