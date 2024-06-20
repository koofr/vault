import { lazy } from 'react';

export const LandingPageOfficialLazy = lazy(() =>
  import('./LandingPageOfficial').then((mod) => ({
    default: mod.LandingPageOfficial,
  })),
);
