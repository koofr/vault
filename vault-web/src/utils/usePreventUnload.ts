import { useEffect } from 'react';

function onBeforeUnload(event: BeforeUnloadEvent) {
  event.preventDefault();
  return '';
}

export function usePreventUnload(prevent: boolean) {
  useEffect(() => {
    if (prevent) {
      window.onbeforeunload = onBeforeUnload;
    } else {
      window.onbeforeunload = null;
    }
  }, [prevent]);

  useEffect(() => {
    return () => {
      window.onbeforeunload = null;
    };
  }, []);
}
