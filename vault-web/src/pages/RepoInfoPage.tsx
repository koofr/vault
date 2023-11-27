import { memo } from 'react';
import { useParams } from 'react-router-dom';

import { RepoInfoComponent } from '../features/repo/RepoInfoComponent';

export const RepoInfoPage = memo(() => {
  const params = useParams();
  const repoId = params.repoId;

  if (repoId === undefined) {
    return null;
  }

  return <RepoInfoComponent key={repoId} repoId={repoId} />;
});
