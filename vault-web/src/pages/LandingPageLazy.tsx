import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const LandingPageLazy = lazyLoadingComponent<{}>(
  () => import('./LandingPage').then((mod) => mod.LandingPage),
  false,
);
