import isEqual from 'lodash/isEqual';
import { useEffect, useMemo, useRef } from 'react';

import { RepoFilesBrowserSource } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

export function useBrowser(
  source: RepoFilesBrowserSource,
  selectName: string | undefined,
): number {
  const webVault = useWebVault();

  const lastSource = useRef<RepoFilesBrowserSource>(source);
  const lastBrowserId = useRef<number>(undefined);

  const browserId = useMemo(() => {
    if (
      // eslint-disable-next-line react-hooks/refs
      lastBrowserId.current !== undefined &&
      // eslint-disable-next-line react-hooks/refs
      isEqual(source, lastSource.current) &&
      selectName === undefined
    ) {
      // if selectName was set and then changed to undefined, use the same
      // browserId
      // eslint-disable-next-line react-hooks/refs
      return lastBrowserId.current;
    }

    const browserId = webVault.repoFilesBrowsersCreate(source, {
      selectName,
    });

    // eslint-disable-next-line react-hooks/refs
    lastSource.current = source;
    // eslint-disable-next-line react-hooks/refs
    lastBrowserId.current = browserId;

    return browserId;
  }, [webVault, source, selectName]);

  useEffect(() => {
    return () => {
      webVault.repoFilesBrowsersDestroy(browserId);
    };
  }, [webVault, browserId]);

  return browserId;
}
