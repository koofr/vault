import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const RepoFilesDetailsPageLazy = lazyLoadingComponent<{}>(
  () =>
    import('./RepoFilesDetailsPage').then(
      (mod) => mod.RepoFilesDetailsPage
    ),
  true
);
