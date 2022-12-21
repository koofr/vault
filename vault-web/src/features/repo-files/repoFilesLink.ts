import { To } from 'react-router-dom';

export const repoFilesLink = (repoId: string, path: string): To => ({
  pathname: `/repos/${repoId}`,
  search: `path=${encodeURIComponent(path)}`,
});
