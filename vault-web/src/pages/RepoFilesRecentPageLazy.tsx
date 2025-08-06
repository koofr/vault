import { lazy } from 'react';

export const RepoFilesRecentPageLazy = lazy(() =>
  import('./RepoFilesRecentPage').then((mod) => ({
    default: mod.RepoFilesRecentPage,
  })),
);
