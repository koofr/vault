import { DependencyList, useEffect, useMemo, useRef, useState } from 'react';

import { WebVault } from '../vault-wasm/vault-wasm';

import { useWebVault } from './useWebVault';

type EraseThis<T> = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [K in keyof T]: T[K] extends (...args: any[]) => any
    ? OmitThisParameter<T[K]>
    : T[K];
};

export function useSubscribe<T>(
  subscribe: (webVault: WebVault, callback: () => void) => number,
  getDataFunc: (webVault: EraseThis<WebVault>) => (subscriptionId: number) => T,
  deps: DependencyList,
): [T, { current: T }] {
  const webVault = useWebVault();

  const depsVersion = useRef(0);
  const currentSubscriptionId = useRef<number>(undefined);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_, setVersion] = useState(0);
  const data = useRef<T>(undefined as T);
  const mountedResolve = useRef<() => void>(undefined);
  const mountedPromise = useRef<Promise<void>>(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    undefined as any as Promise<void>,
  );
  if (mountedPromise.current === undefined) {
    mountedPromise.current = new Promise<void>((resolve) => {
      mountedResolve.current = resolve;
    });
  }
  useEffect(() => {
    mountedResolve.current?.();
  });

  useMemo(() => {
    // if the deps have changed, increase the version so that we can ignore
    // stale subscribe callbacks
    const lastDepsVersion = depsVersion.current + 1;
    depsVersion.current = lastDepsVersion;

    if (currentSubscriptionId.current !== undefined) {
      webVault.unsubscribe(currentSubscriptionId.current);
    }

    let subscriptionId: number | undefined;

    const getData = getDataFunc(webVault);

    // eslint-disable-next-line prefer-const
    subscriptionId = subscribe(webVault, () => {
      if (lastDepsVersion !== depsVersion.current) {
        // lastDepsVersion !== depsVersion.current, the deps have changed and
        // we have unsubscribed the last subscription so getData would return
        // undefined
        return;
      }

      data.current = getData.call(webVault, subscriptionId!);

      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      mountedPromise.current.then(() => {
        setVersion((version) => version + 1);
      });
    });

    currentSubscriptionId.current = subscriptionId;

    const initialData = getData.call(webVault, subscriptionId);

    data.current = initialData;
    // eslint-disable-next-line react-hooks/use-memo, react-hooks/exhaustive-deps
  }, [webVault, ...deps]);

  useEffect(() => {
    return () => {
      if (currentSubscriptionId.current !== undefined) {
        webVault.unsubscribe(currentSubscriptionId.current);
      }
    };
  }, [webVault]);

  return [data.current, data];
}
