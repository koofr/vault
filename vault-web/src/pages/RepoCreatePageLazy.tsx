import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const RepoCreatePageLazy = lazyLoadingComponent<{}>(
  () => import('./RepoCreatePage').then((mod) => mod.RepoCreatePage),
  true,
);
