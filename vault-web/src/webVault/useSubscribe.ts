import { DependencyList, useEffect, useMemo, useRef, useState } from 'react';

import { WebVault } from '../vault-wasm/vault-wasm';

import { useWebVault } from './useWebVault';

export function useSubscribe<T>(
  subscribe: (webVault: WebVault, callback: () => void) => number,
  getDataFunc: (webVault: WebVault) => (subscriptionId: number) => T,
  deps: DependencyList
): T {
  const webVault = useWebVault();

  const currentSubscriptionId = useRef<number>();
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_, setVersion] = useState(0);
  const data = useRef<T>();

  useMemo(
    () => {
      if (currentSubscriptionId.current !== undefined) {
        webVault.unsubscribe(currentSubscriptionId.current);
      }

      let subscriptionId: number | undefined;

      const getData = getDataFunc(webVault);

      subscriptionId = subscribe(webVault, () => {
        data.current = getData.call(webVault, subscriptionId!);
        setVersion((version) => version + 1);
      });

      currentSubscriptionId.current = subscriptionId;

      const initialData = getData.call(webVault, subscriptionId!);

      // eslint-disable-next-line react-hooks/exhaustive-deps
      data.current = initialData;
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [webVault, ...deps]
  );

  useEffect(() => {
    return () => {
      if (currentSubscriptionId.current !== undefined) {
        webVault.unsubscribe(currentSubscriptionId.current);
      }
    };
  }, [webVault]);

  return data.current!;
}
