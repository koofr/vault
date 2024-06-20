import { lazy } from 'react';

export const OAuth2CallbackPageLazy = lazy(() =>
  import('./OAuth2CallbackPage').then((mod) => ({
    default: mod.OAuth2CallbackPage,
  })),
);
