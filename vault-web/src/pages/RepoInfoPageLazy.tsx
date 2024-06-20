import { lazy } from 'react';

export const RepoInfoPageLazy = lazy(() =>
  import('./RepoInfoPage').then((mod) => ({ default: mod.RepoInfoPage })),
);
