import { lazyLoadingComponent } from '../../components/lazyLoadingComponent';

export const RepoFilesDropZoneDesktopLazy = lazyLoadingComponent<{}>(
  () =>
    import('./RepoFilesDropZoneDesktop').then(
      (mod) => mod.RepoFilesDropZoneDesktop,
    ),
  false,
);
