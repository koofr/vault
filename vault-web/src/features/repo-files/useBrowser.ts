import { useEffect, useMemo, useRef } from 'react';

import { useWebVault } from '../../webVault/useWebVault';

export function useBrowser(
  repoId: string,
  encryptedPath: string,
  selectName: string | undefined,
): number {
  const webVault = useWebVault();

  const lastRepoId = useRef<string>(repoId);
  const lastEncryptedPath = useRef<string>(encryptedPath);
  const lastBrowserId = useRef<number>();

  const browserId = useMemo(() => {
    if (
      lastBrowserId.current !== undefined &&
      repoId === lastRepoId.current &&
      encryptedPath === lastEncryptedPath.current &&
      selectName === undefined
    ) {
      // if selectName was set and then changed to undefined, use the same
      // browserId
      return lastBrowserId.current;
    }

    const browserId = webVault.repoFilesBrowsersCreate(repoId, encryptedPath, {
      selectName,
    });

    lastRepoId.current = repoId;
    lastEncryptedPath.current = encryptedPath;
    lastBrowserId.current = browserId;

    return browserId;
  }, [webVault, repoId, encryptedPath, selectName]);

  useEffect(() => {
    return () => {
      webVault.repoFilesBrowsersDestroy(browserId);
    };
  }, [webVault, browserId]);

  return browserId;
}
