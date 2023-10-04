import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const NotFoundPageLazy = lazyLoadingComponent<{}>(
  () => import('./NotFoundPage').then((mod) => mod.NotFoundPage),
  true,
);
