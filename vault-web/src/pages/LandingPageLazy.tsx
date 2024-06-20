import { lazy } from 'react';

export const LandingPageLazy = lazy(() =>
  import('./LandingPage').then((mod) => ({ default: mod.LandingPage })),
);
