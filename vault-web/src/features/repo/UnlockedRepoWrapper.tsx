import { PropsWithChildren, memo } from 'react';

import { useLockRepoShortcut } from './useLockRepoShortcut';

export const UnlockedRepoWrapper = memo<PropsWithChildren<{ repoId: string }>>(
  ({ repoId, children }) => {
    useLockRepoShortcut(repoId);

    return children;
  },
);
