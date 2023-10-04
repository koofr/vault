import { css, cx } from '@emotion/css';
import { memo } from 'react';

import { Repos } from '../../features/repos/Repos';

import { useIsMobile } from '../useIsMobile';

import { DashboardSidenavLinks } from './DashboardSidenavLinks';

export const DashboardSidenav = memo<{ className?: string }>(
  ({ className }) => {
    const isMobile = useIsMobile();

    return (
      <div
        className={cx(
          css``,
          isMobile
            ? css`
                width: 100%;
                margin: 0;
              `
            : css`
                width: 225px;
                flex-grow: 0;
                flex-shrink: 0;
                margin: 0 50px 0 0;
                overflow: hidden;
              `,
          className,
        )}
      >
        <Repos />

        <DashboardSidenavLinks />
      </div>
    );
  },
);
