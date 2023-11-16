import { useEffect, useMemo, useRef } from 'react';

import { useWebVault } from '../../webVault/useWebVault';

export function useBrowser(
  repoId: string,
  path: string,
  selectName: string | undefined,
): number {
  const webVault = useWebVault();

  const lastRepoId = useRef<string>(repoId);
  const lastPath = useRef<string>(path);
  const lastBrowserId = useRef<number>();

  const browserId = useMemo(() => {
    if (
      lastBrowserId.current !== undefined &&
      repoId === lastRepoId.current &&
      path === lastPath.current &&
      selectName === undefined
    ) {
      // if selectName was set and then changed to undefined, use the same
      // browserId
      return lastBrowserId.current;
    }

    const browserId = webVault.repoFilesBrowsersCreate(repoId, path, {
      selectName,
    });

    lastRepoId.current = repoId;
    lastPath.current = path;
    lastBrowserId.current = browserId;

    return browserId;
  }, [webVault, repoId, path, selectName]);

  useEffect(() => {
    return () => {
      webVault.repoFilesBrowsersDestroy(browserId);
    };
  }, [webVault, browserId]);

  return browserId;
}
