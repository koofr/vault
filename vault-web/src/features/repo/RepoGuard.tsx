import React from 'react';

import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { useSubscribe } from '../../webVault/useSubscribe';

import { RepoError } from './RepoError';
import { RepoUnlock } from './RepoUnlock';
import { useWebVault } from '../../webVault/useWebVault';

export const RepoGuard: React.FC<{
  repoId: string;
  component: React.ComponentType<{ repoId: string }>;
}> = ({ repoId, component }) => {
  const webVault = useWebVault();
  const [info] = useSubscribe(
    (v, cb) => v.reposRepoSubscribe(repoId, cb),
    (v) => v.reposRepoData,
    [repoId],
  );

  if (info.repo !== undefined) {
    return info.repo.state === 'Locked' ? (
      <RepoUnlock key={repoId} repoId={info.repo.id} />
    ) : (
      React.createElement(component, { repoId: info.repo.id })
    );
  } else if (info.status.type === 'Error') {
    return (
      <RepoError error={info.status.error} onRetry={() => webVault.load()} />
    );
  } else {
    return <DashboardLoading />;
  }
};
