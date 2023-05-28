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

export const getNavbarHeight = (isMobile: boolean) =>
  isMobile ? NAVBAR_HEIGHT_MOBILE : NAVBAR_HEIGHT;

export const NavbarMain = memo<{
  left?: ReactNode;
  header?: ReactNode;
  nav?: ReactNode;
  right?: ReactNode;
  isSticky: boolean;
  noShadow?: boolean;
}>(({ left, header, nav, right, isSticky, noShadow }) => {
  const theme = useTheme();

  return (
    <header
      className={cx(
        css`
          display: flex;
          align-items: center;
          position: sticky;
          width: 100%;
          top: 0;
          z-index: ${theme.zindex.navbarMain};
          background-color: ${theme.colors.navbarBg};
          box-shadow: ${noShadow
            ? `none`
            : `0 1px 3px 0 ${theme.colors.border}`};
          border-bottom: 1px solid
            ${noShadow ? theme.colors.border : 'transparent'};
          height: ${getNavbarHeight(theme.isMobile)}px;
          padding: 0 ${theme.isMobile ? theme.gutterMobile : theme.gutter};
          overflow: hidden;
        `,
        isSticky
          ? css`
              box-shadow: none;
              border-bottom: 1px solid ${theme.colors.borderLight};
            `
          : undefined
      )}
      aria-label="Navbar"
    >
      <div
        className={css`
          display: flex;
          align-items: center;
          flex-shrink: 0;
          width: 32px;
          height: 32px;
        `}
        aria-label="Navbar left"
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
        className={cx(
          css`
            flex-grow: 1;
            display: flex;
            align-items: center;
            overflow-x: auto;
          `
        )}
      >
        <div
          className={css`
            font-size: 14px;
            font-weight: normal;
            color: ${theme.colors.text};
            text-overflow: ellipsis;
            white-space: nowrap;
            overflow: hidden;
            flex-grow: 1;
            flex-shrink: 0;
          `}
          aria-label="Navbar header"
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
            aria-label="Navbar nav"
          >
            {nav}
          </div>
        ) : null}
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
          display: flex;
          align-items: center;
          flex-shrink: 0;
          width: 32px;
          height: 32px;
        `}
        aria-label="Navbar right"
      >
        {right}
      </div>
    </header>
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
              top: ${getNavbarHeight(theme.isMobile)}px;
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
                  // TODO searchbox
                  padding-left: ${theme.isMobile ? '15px' : '275px'};
                `
              : css`
                  // TODO searchbox
                  margin-left: ${theme.isMobile ? '0' : '250px'};
                `,
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
  noShadow?: boolean;
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
