import { PropsWithChildren, memo } from 'react';

import { DashboardError } from '../../components/dashboard/DashboardError';
import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { Status } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoUnlock } from './RepoUnlock';
import { UnlockedRepoWrapper } from './UnlockedRepoWrapper';

export const RepoGuard = memo<
  PropsWithChildren<{ repoId: string; repoStatus: Status; isLocked: boolean }>
>(({ repoId, repoStatus, isLocked, children }) => {
  const webVault = useWebVault();

  if (
    repoStatus.type === 'Initial' ||
    (repoStatus.type === 'Loading' && !repoStatus.loaded)
  ) {
    return <DashboardLoading />;
  } else if (repoStatus.type === 'Error') {
    return (
      <DashboardError
        error={repoStatus.error}
        onRetry={() => {
          webVault.load();
        }}
      />
    );
  } else if (isLocked) {
    return <RepoUnlock key={repoId} repoId={repoId} />;
  } else {
    return (
      <UnlockedRepoWrapper repoId={repoId}>{children}</UnlockedRepoWrapper>
    );
  }
});
