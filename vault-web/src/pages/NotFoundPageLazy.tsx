import { lazy } from 'react';

export const NotFoundPageLazy = lazy(() =>
  import('./NotFoundPage').then((mod) => ({ default: mod.NotFoundPage })),
);
