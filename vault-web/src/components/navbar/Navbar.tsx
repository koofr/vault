import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, PropsWithChildren, ReactNode, useMemo } from 'react';

import { useDocumentScroll } from '../DocumentScroll';
import { useIsMobile } from '../useIsMobile';

import { NavbarLogo } from './NavbarLogo';

export const NAVBAR_HEIGHT = 70;
export const NAVBAR_HEIGHT_MOBILE = 60;
export const NAVBAR_STICKY_SCROLL = 25;
export const NAVBAR_STICKY_SCROLL_MOBILE = 1;

export const NavbarMain = memo<{
  left?: ReactNode;
  header?: ReactNode;
  nav?: ReactNode;
  right?: ReactNode;
  isSticky: boolean;
}>(({ left, header, nav, right, isSticky }) => {
  const theme = useTheme();

  return (
    <div
      className={cx(
        css`
          display: flex;
          align-items: center;
          position: sticky;
          width: 100%;
          top: 0;
          z-index: ${theme.zindex.navbarMain};
          background-color: ${theme.colors.navbarBg};
          box-shadow: 0 1px 3px 0 ${theme.colors.border};
          border-bottom: 1px solid transparent;
          height: ${theme.isMobile
            ? `${NAVBAR_HEIGHT_MOBILE}px`
            : `${NAVBAR_HEIGHT}px`};
          padding: 0 ${theme.isMobile ? theme.gutterMobile : theme.gutter};
        `,
        isSticky
          ? css`
              box-shadow: none;
              border-bottom: 1px solid ${theme.colors.borderLight};
            `
          : undefined
      )}
    >
      <div
        className={css`
          display: flex;
          align-items: center;
          flex-shrink: 0;
          width: 32px;
          height: 32px;
        `}
      >
        {left ?? <NavbarLogo />}
      </div>
      <div
        className={css`
          width: 0px;
          height: 30px;
          border-right: 1px solid ${theme.colors.navbarVerticalLine};
          margin: 0 ${theme.isMobile ? theme.gutterMobile : theme.gutter};
        `}
      ></div>
      <div
        className={css`
          font-size: 14px;
          font-weight: normal;
          color: ${theme.colors.text};
          text-overflow: ellipsis;
          white-space: nowrap;
          overflow: hidden;
          flex-grow: 1;
        `}
      >
        {header}
      </div>
      {nav !== undefined ? (
        <div
          className={css`
            flex-shrink: 0;
            display: flex;
            align-items: center;
          `}
        >
          {nav}
        </div>
      ) : null}
      <div
        className={css`
          width: 0px;
          height: 30px;
          border-right: 1px solid ${theme.colors.navbarVerticalLine};
          margin: 0 ${theme.isMobile ? theme.gutterMobile : theme.gutter};
        `}
      ></div>
      <div
        className={css`
          display: flex;
          align-items: center;
          flex-shrink: 0;
          width: 32px;
          height: 32px;
        `}
      >
        {right}
      </div>
    </div>
  );
});

export const NavbarExtra = memo<PropsWithChildren<{ isSticky: boolean }>>(
  ({ isSticky, children }) => {
    const theme = useTheme();

    return (
      <>
        <div
          className={css`
            height: ${theme.isMobile ? '0' : '25px'};
          `}
        ></div>
        <div
          className={cx(
            css`
              position: sticky;
              top: ${theme.isMobile
                ? `${NAVBAR_HEIGHT_MOBILE}px`
                : `${NAVBAR_HEIGHT}px`};
              padding: 0 ${theme.isMobile ? theme.gutterMobile : theme.gutter};
              height: 45px;
              display: flex;
              align-items: center;
            `,
            isSticky
              ? css`
                  box-shadow: 0 1px 3px 0 ${theme.colors.border};
                  background-color: #fff;
                  z-index: ${theme.zindex.navbarExtra};
                `
              : undefined,
            theme.isMobile
              ? css`
                  overflow-x: auto;
                `
              : css``
          )}
        >
          {children}
        </div>
      </>
    );
  }
);

export const Navbar = memo<{
  left?: ReactNode;
  header?: ReactNode;
  nav?: ReactNode;
  right?: ReactNode;
  extra?: ReactNode;
}>(({ extra, ...props }) => {
  const isMobile = useIsMobile();
  const scrollInfo = useDocumentScroll();
  const stickyScroll = isMobile
    ? NAVBAR_STICKY_SCROLL_MOBILE
    : NAVBAR_STICKY_SCROLL;
  const isSticky = useMemo(
    () => scrollInfo.y >= stickyScroll,
    [scrollInfo.y, stickyScroll]
  );

  return (
    <>
      <NavbarMain {...props} isSticky={isSticky} />
      {extra !== undefined ? (
        <NavbarExtra isSticky={isSticky}>{extra}</NavbarExtra>
      ) : undefined}
    </>
  );
});
