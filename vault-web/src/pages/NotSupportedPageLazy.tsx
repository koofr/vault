import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const NotSupportedPageLazy = lazyLoadingComponent<{}>(
  () => import('./NotSupportedPage').then((mod) => mod.NotSupportedPage),
  true
);
