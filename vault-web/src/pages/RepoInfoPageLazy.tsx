import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const RepoInfoPageLazy = lazyLoadingComponent<{}>(
  () => import('./RepoInfoPage').then((mod) => mod.RepoInfoPage),
  true,
);
