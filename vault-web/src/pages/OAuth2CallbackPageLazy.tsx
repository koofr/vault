import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const OAuth2CallbackPageLazy = lazyLoadingComponent<{}>(
  () => import('./OAuth2CallbackPage').then((mod) => mod.OAuth2CallbackPage),
  true
);
