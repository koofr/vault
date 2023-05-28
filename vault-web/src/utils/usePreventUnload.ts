import { useEffect, useRef } from 'react';

function onBeforeUnload(event: BeforeUnloadEvent) {
  event.preventDefault();
  return '';
}

let preventCount = 0;

function onPreventCountChange() {
  if (preventCount === 1) {
    window.onbeforeunload = onBeforeUnload;
  } else if (preventCount === 0) {
    window.onbeforeunload = null;
  }
}

export function usePreventUnload(prevent: boolean) {
  const oldPrevent = useRef(false);

  useEffect(() => {
    if (prevent && !oldPrevent.current) {
      preventCount++;
      oldPrevent.current = true;
    } else if (!prevent && oldPrevent.current) {
      preventCount--;
      oldPrevent.current = false;
    }

    onPreventCountChange();
  }, [prevent]);

  useEffect(() => {
    return () => {
      if (oldPrevent.current) {
        preventCount--;

        onPreventCountChange();
      }
    };
  }, []);
}
