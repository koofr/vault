import { css, cx } from '@emotion/css';
import { memo, PropsWithChildren, ReactNode } from 'react';

import { SpaceUsage } from '../../features/space-usage/SpaceUsage';
import { Uploads } from '../../features/uploads/Uploads';

import { useIsMobile } from '../useIsMobile';

import { DashboardNavbar } from './DashboardNavbar';
import { DashboardSidenav } from './DashboardSidenav';

export const DashboardLayout = memo<
  PropsWithChildren<{
    navbarLeft?: ReactNode;
    navbarHeader?: ReactNode;
    navbarNav?: ReactNode;
    navbarExtra?: ReactNode;
    className?: string;
    sidenavClassName?: string;
  }>
>(
  ({
    navbarLeft,
    navbarHeader,
    navbarNav,
    navbarExtra,
    className,
    sidenavClassName,
    children,
  }) => {
    const isMobile = useIsMobile();

    return (
      <>
        <DashboardNavbar
          left={navbarLeft}
          header={navbarHeader}
          nav={navbarNav}
          extra={navbarExtra}
        />
        <div
          className={cx(
            css`
              display: flex;
            `,
            isMobile
              ? css`
                  flex-direction: column-reverse;
                  padding: 30px 0 75px;
                `
              : css`
                  flex-direction: row;
                  flex-grow: 1;
                  padding: 30px 25px 75px 0;
                `,
            className
          )}
        >
          <DashboardSidenav className={sidenavClassName} />
          <div
            className={cx(
              css`
                display: flex;
                flex-direction: column;
                flex-grow: 1;
                overflow-x: hidden;
              `,
              isMobile
                ? css`
                    margin: 0 0 32px;
                  `
                : css`
                    margin: 0 32px 0 0;
                  `
            )}
          >
            {children}
          </div>
        </div>

        <Uploads />
        <SpaceUsage />
      </>
    );
  }
);
