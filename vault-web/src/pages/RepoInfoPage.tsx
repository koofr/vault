import { memo } from 'react';
import { useParams } from 'react-router-dom';

import { RepoInfo } from '../features/repo/RepoInfo';

export const RepoInfoPage = memo(() => {
  const params = useParams();
  const repoId = params.repoId;

  if (repoId === undefined) {
    return null;
  }

  return <RepoInfo key={repoId} repoId={repoId} />;
});
