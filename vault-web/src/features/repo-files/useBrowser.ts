import { useEffect, useMemo, useRef } from 'react';

import { useWebVault } from '../../webVault/useWebVault';

export function useBrowser(
  repoId: string,
  path: string,
  selectName: string | undefined
): number {
  const webVault = useWebVault();

  const browserId = useMemo(
    () => {
      // we create a new browser with repoId and path and then use setLocation
      // to update path
      return webVault.repoFilesBrowsersCreate(repoId, path, {
        selectName: selectName !== '' ? selectName : undefined,
      });
    },
    // ignore selectName changes, ignore navigate changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [webVault]
  );

  useEffect(() => {
    return () => {
      webVault.repoFilesBrowsersDestroy(browserId);
    };
  }, [webVault, browserId]);

  const setLocationCount = useRef(0);

  useEffect(() => {
    if (setLocationCount.current > 0) {
      webVault.repoFilesBrowsersSetLocation(browserId, repoId, path);
    }

    setLocationCount.current += 1;
  }, [webVault, browserId, repoId, path]);

  return browserId;
}
