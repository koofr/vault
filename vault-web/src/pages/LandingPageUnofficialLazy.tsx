import { lazy } from 'react';

export const LandingPageUnofficialLazy = lazy(() =>
  import('./LandingPageUnofficial').then((mod) => ({
    default: mod.LandingPageUnofficial,
  })),
);
