import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const MobilePageLazy = lazyLoadingComponent<{}>(
  () => import('./MobilePage').then((mod) => mod.MobilePage),
  true,
);
