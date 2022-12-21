import { useEffect, useRef } from 'react';

export function useAutofocus() {
  const ref = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    ref.current?.focus();
  }, [ref]);

  return ref;
}
