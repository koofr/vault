import React from 'react';

import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';

import { RepoUnlock } from './RepoUnlock';

export const RepoGuard: React.FC<{
  repoId: string;
  component: React.ComponentType<{ repo: Repo }>;
}> = ({ repoId, component }) => {
  const repo = useSubscribe(
    (v, cb) => v.reposRepoSubscribe(repoId, cb),
    (v) => v.reposRepoData,
    [repoId]
  );

  if (repo === undefined) {
    return <DashboardLoading />;
  }

  return repo.state === 'Locked' ? (
    <RepoUnlock key={repoId} repoId={repo.id} />
  ) : (
    React.createElement(component, { repo })
  );
};
