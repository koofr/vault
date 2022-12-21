import { useCallback, useState } from 'react';

export function useModal<Payload = void>(): {
  isVisible: boolean;
  payload: Payload | undefined;
  show: (payload: Payload) => void;
  hide: () => void;
} {
  const [state, setState] = useState<{
    isVisible: boolean;
    payload: Payload | undefined;
  }>({ isVisible: false, payload: undefined });
  const show = useCallback(
    (payload: Payload) => setState({ isVisible: true, payload }),
    []
  );
  const hide = useCallback(
    () => setState({ isVisible: false, payload: undefined }),
    []
  );

  return { isVisible: state.isVisible, payload: state.payload, show, hide };
}
