import { memo } from 'react';
import { useParams } from 'react-router-dom';

import { RepoFiles } from '../features/repo-files/RepoFiles';

export const RepoFilesPage = memo(() => {
  const params = useParams();
  const repoId = params.repoId;

  if (repoId === undefined) {
    return null;
  }

  return <RepoFiles repoId={repoId} />;
});
