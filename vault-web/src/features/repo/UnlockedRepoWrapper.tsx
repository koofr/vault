import { PropsWithChildren, memo } from 'react';

import { useLockRepoShortcut } from './useLockRepoShortcut';
import { useTouchRepo } from './useTouchRepo';

export const UnlockedRepoWrapper = memo<PropsWithChildren<{ repoId: string }>>(
  ({ repoId, children }) => {
    useLockRepoShortcut(repoId);
    useTouchRepo(repoId);

    return children;
  },
);
