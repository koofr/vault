import { lazy } from 'react';

export const RepoFilesDetailsPageLazy = lazy(() =>
  import('./RepoFilesDetailsPage').then((mod) => ({
    default: mod.RepoFilesDetailsPage,
  })),
);
