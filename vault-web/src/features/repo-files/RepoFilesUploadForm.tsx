import { memo } from 'react';

import { RepoFilesUploadFormDesktop } from './RepoFilesUploadFormDesktop';
import { RepoFilesUploadFormWeb } from './RepoFilesUploadFormWeb';

export const RepoFilesUploadForm = memo(() => {
  if (import.meta.env.VITE_VAULT_APP === 'desktop') {
    return <RepoFilesUploadFormDesktop />;
  } else {
    return <RepoFilesUploadFormWeb />;
  }
});
