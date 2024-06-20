import { lazy } from 'react';

export const HomePageLazy = lazy(() =>
  import('./HomePage').then((mod) => ({ default: mod.HomePage })),
);
