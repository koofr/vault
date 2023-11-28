import { memo } from 'react';
import { useParams } from 'react-router-dom';

import { RepoFilesDetails } from '../features/repo-files-details/RepoFilesDetails';

export const RepoFilesDetailsPage = memo(() => {
  const params = useParams();
  const repoId = params.repoId;

  if (repoId === undefined) {
    return null;
  }

  return <RepoFilesDetails repoId={repoId} />;
});
