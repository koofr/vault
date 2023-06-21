import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const RepoConfigBackupPageLazy = lazyLoadingComponent<{}>(
  () => import('./RepoConfigBackupPage').then((mod) => mod.RepoConfigBackupPage),
  true
);
