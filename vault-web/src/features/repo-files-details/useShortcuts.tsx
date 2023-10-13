import { useEffect } from 'react';

import { isMacOS } from '../../utils/browser';
import { isEventKeys } from '../../utils/keyboardEvents';
import { RepoFilesDetailsInfo } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

export function useShortcuts(
  detailsId: number,
  infoRef: { current: RepoFilesDetailsInfo | undefined },
) {
  const webVault = useWebVault();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        (isMacOS && isEventKeys(event, 's', { metaKey: true })) ||
        (!isMacOS && isEventKeys(event, 's', { ctrlKey: true }))
      ) {
        event.preventDefault();

        if (infoRef.current !== undefined && infoRef.current.canSave) {
          webVault.repoFilesDetailsSave(detailsId);
        }
      }
    };

    document.addEventListener('keydown', onKeyDown);

    return () => {
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [webVault, detailsId, infoRef]);
}
