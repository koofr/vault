import { memo } from 'react';
import { useParams } from 'react-router-dom';

import { RepoConfigBackup } from '../features/repo/RepoConfigBackup';

export const RepoConfigBackupPage = memo(() => {
  const params = useParams();
  const repoId = params.repoId;

  if (repoId === undefined) {
    return null;
  }

  return <RepoConfigBackup key={repoId} repoId={repoId} />;
});
