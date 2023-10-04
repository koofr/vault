import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const HomePageLazy = lazyLoadingComponent<{}>(
  () => import('./HomePage').then((mod) => mod.HomePage),
  true,
);
