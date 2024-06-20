import { Suspense, memo } from 'react';

import { RepoFilesDropZoneDesktopLazy } from './RepoFilesDropZoneDesktopLazy';
import { RepoFilesDropZoneWeb } from './RepoFilesDropZoneWeb';

export const RepoFilesDropZone = memo(() => {
  if (import.meta.env.VITE_VAULT_APP === 'desktop') {
    return (
      <Suspense>
        <RepoFilesDropZoneDesktopLazy />
      </Suspense>
    );
  } else {
    return <RepoFilesDropZoneWeb />;
  }
});
