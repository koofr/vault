import { lazy } from 'react';

export const RepoFilesDropZoneDesktopLazy = lazy(() =>
  import('./RepoFilesDropZoneDesktop').then((mod) => ({
    default: mod.RepoFilesDropZoneDesktop,
  })),
);
