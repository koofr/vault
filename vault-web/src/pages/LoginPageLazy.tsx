import { lazy } from 'react';

export const LoginPageLazy = lazy(() =>
  import('./LoginPage').then((mod) => ({ default: mod.LoginPage })),
);
