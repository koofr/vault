import { UsePopperState } from '@restart/ui/usePopper';
import { useLayoutEffect } from 'react';

export function useMenuUpdate(show: boolean, popper: UsePopperState | null) {
  useLayoutEffect(() => {
    if (show && popper !== null) {
      popper.update();
    }
    // useLayoutEffect must not depend on popper as this causes an endless loop
    // because popper object changes on every render
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [show]);
}
