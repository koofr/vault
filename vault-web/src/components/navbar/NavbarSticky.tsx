import { createContext, PropsWithChildren, useContext } from 'react';

import { useDocumentScroll } from '../DocumentScroll';
import { useIsMobile } from '../useIsMobile';

export const NAVBAR_STICKY_SCROLL = 25;
export const NAVBAR_STICKY_SCROLL_MOBILE = 1;

export const NavbarStickyContext = createContext<boolean>(false);

export const NavbarStickyProvider: React.FC<PropsWithChildren<{}>> = ({
  children,
}) => {
  const isMobile = useIsMobile();
  const scrollInfo = useDocumentScroll();
  const stickyScroll = isMobile
    ? NAVBAR_STICKY_SCROLL_MOBILE
    : NAVBAR_STICKY_SCROLL;
  const isSticky = scrollInfo.y >= stickyScroll;

  return (
    <NavbarStickyContext.Provider value={isSticky}>
      {children}
    </NavbarStickyContext.Provider>
  );
};

export function useNavbarSticky(): boolean {
  const isSticky = useContext(NavbarStickyContext);

  return isSticky;
}
