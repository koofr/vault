import { lazy } from 'react';

export const RepoCreatePageLazy = lazy(() =>
  import('./RepoCreatePage').then((mod) => ({ default: mod.RepoCreatePage })),
);
