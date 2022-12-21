import * as React from 'react';

import { LoadingCircle } from './LoadingCircle';

const onImportError = (err: any) => {
  console.warn('Import failed:', err);

  alert('Something went wrong. Please refresh the page.');
};

export function lazyLoadingComponent<P>(
  load: () => Promise<React.ComponentType<P>>,
  loadingCircle = true
) {
  let isLoaded = false;
  let isLoading = false;
  let component: React.ComponentType<P> | undefined;
  let loadPromise: Promise<void> | undefined;

  const cachedLoad = (): Promise<void> | undefined => {
    if (isLoaded) {
      return;
    }

    if (isLoading) {
      return loadPromise;
    }

    isLoading = true;

    loadPromise = load().then((loadedComponent) => {
      component = loadedComponent;
      isLoaded = true;
      isLoading = false;
    }, onImportError);

    return loadPromise;
  };

  const WrapperComponent: React.FC<any> = (props) => {
    const isMountedRef = React.useRef(true);
    const [, setDummy] = React.useState(false);
    React.useEffect(() => {
      const promiseO = cachedLoad();
      if (promiseO !== undefined) {
        promiseO.then(() => {
          if (isMountedRef.current) {
            // force render
            setDummy(true);
          }
        });
      } else {
        setDummy(true);
      }

      return () => {
        isMountedRef.current = false;
      };
    }, []);

    if (!isLoaded) {
      if (loadingCircle) {
        return <LoadingCircle />;
      }

      return null;
    }

    return React.createElement(component! as any, props, null);
  };

  (WrapperComponent as any).preload = cachedLoad;

  return WrapperComponent;
}
