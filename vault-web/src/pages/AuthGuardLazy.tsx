import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const AuthGuardLazy = lazyLoadingComponent<{}>(
  () => import('./AuthGuard').then((mod) => mod.AuthGuard),
  true
);
