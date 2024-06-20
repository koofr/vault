import { lazy } from 'react';

export const RepoFilesPageLazy = lazy(() =>
  import('./RepoFilesPage').then((mod) => ({ default: mod.RepoFilesPage })),
);
