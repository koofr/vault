import React from 'react';

import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';

import { RepoError } from './RepoError';
import { RepoUnlock } from './RepoUnlock';

export const RepoGuard: React.FC<{
  repoId: string;
  component: React.ComponentType<{ repo: Repo }>;
}> = ({ repoId, component }) => {
  const info = useSubscribe(
    (v, cb) => v.reposRepoSubscribe(repoId, cb),
    (v) => v.reposRepoData,
    [repoId]
  );

  if (info.status.type === 'Error') {
    return <RepoError error={info.status.error} />;
  } else if (info.repo !== undefined) {
    return info.repo.state === 'Locked' ? (
      <RepoUnlock key={repoId} repoId={info.repo.id} />
    ) : (
      React.createElement(component, { repo: info.repo })
    );
  } else if (
    info.status.type === 'Loading' ||
    info.status.type === 'Reloading'
  ) {
    return <DashboardLoading />;
  }

  return null;
};
