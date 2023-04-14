import { memo, ReactNode } from 'react';

import { Navbar } from '../navbar/Navbar';

import { DashboardUserIconDropdown } from './DashboardUserIconDropdown';

export const DashboardNavbar = memo<{
  left?: ReactNode;
  header?: ReactNode;
  nav?: ReactNode;
  right?: ReactNode;
  extra?: ReactNode;
  noShadow?: boolean;
}>(({ left, header, nav, right, extra, noShadow }) => {
  return (
    <Navbar
      left={left}
      header={header ?? <span>Koofr Vault</span>}
      nav={nav}
      right={right ?? <DashboardUserIconDropdown />}
      extra={extra}
      noShadow={noShadow}
    />
  );
});
