import { useEffect } from 'react';

import { isEventKeys } from '../../utils/keyboardEvents';
import { useWebVault } from '../../webVault/useWebVault';

export function useLockRepoShortcut(repoId: string) {
  const webVault = useWebVault();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (isEventKeys(event, 'l', { altKey: true, shiftKey: true })) {
        event.preventDefault();

        webVault.reposLockRepo(repoId);
      }
    };

    document.addEventListener('keydown', onKeyDown);

    return () => {
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [webVault, repoId]);
}
