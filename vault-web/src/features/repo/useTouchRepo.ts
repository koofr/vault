import { useCallback, useEffect, useMemo } from 'react';

import { useWebVault } from '../../webVault/useWebVault';

const touchIntervalMs = 1000;

export function useTouchRepo(repoId: string) {
  const webVault = useWebVault();

  const touchRepo = useCallback(() => {
    webVault.reposTouchRepo(repoId);
  }, [webVault, repoId]);

  const onInteraction = useMemo(() => {
    let lastInteraction: number = 0;
    let timeoutId: ReturnType<typeof setTimeout> | undefined;

    return function () {
      const currentTime = new Date().getTime();

      clearTimeout(timeoutId);

      if (currentTime - lastInteraction >= touchIntervalMs) {
        touchRepo();

        lastInteraction = currentTime;
      } else {
        timeoutId = setTimeout(() => {
          touchRepo();

          lastInteraction = currentTime;
        }, touchIntervalMs);
      }
    };
  }, [touchRepo]);

  useEffect(() => {
    window.addEventListener('mousemove', onInteraction);
    window.addEventListener('mousedown', onInteraction);
    window.addEventListener('keydown', onInteraction);

    return () => {
      window.removeEventListener('mousemove', onInteraction);
      window.removeEventListener('mousedown', onInteraction);
      window.removeEventListener('keydown', onInteraction);
    };
  }, [onInteraction]);

  useEffect(() => {
    touchRepo();
  }, [touchRepo]);
}
