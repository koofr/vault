import { lazy } from 'react';

export const MobilePageLazy = lazy(() =>
  import('./MobilePage').then((mod) => ({ default: mod.MobilePage })),
);
