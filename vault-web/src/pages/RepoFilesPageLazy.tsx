import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const RepoFilesPageLazy = lazyLoadingComponent<{}>(
  () => import('./RepoFilesPage').then((mod) => mod.RepoFilesPage),
  true,
);
