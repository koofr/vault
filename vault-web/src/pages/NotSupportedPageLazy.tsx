import { lazy } from 'react';

export const NotSupportedPageLazy = lazy(() =>
  import('./NotSupportedPage').then((mod) => ({
    default: mod.NotSupportedPage,
  })),
);
