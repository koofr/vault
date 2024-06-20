import { lazy } from 'react';

export const RepoConfigBackupPageLazy = lazy(() =>
  import('./RepoConfigBackupPage').then((mod) => ({
    default: mod.RepoConfigBackupPage,
  })),
);
