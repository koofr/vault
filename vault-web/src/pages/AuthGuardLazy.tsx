import { lazy } from 'react';

export const AuthGuardLazy = lazy(() =>
  import('./AuthGuard').then((mod) => ({ default: mod.AuthGuard })),
);
