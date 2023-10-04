import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const LoginPageLazy = lazyLoadingComponent<{}>(
  () => import('./LoginPage').then((mod) => mod.LoginPage),
  false,
);
