import { memo, ReactNode } from 'react';

import { Navbar } from '../navbar/Navbar';
import { useSingleNavbarBreadcrumb } from '../navbar/useSingleNavbarBreadcrumb';

import { DashboardUserIconDropdown } from './DashboardUserIconDropdown';

export const DashboardNavbar = memo<{
  left?: ReactNode;
  header?: ReactNode;
  nav?: ReactNode;
  right?: ReactNode;
  extra?: ReactNode;
  noShadow?: boolean;
}>(({ left, header, nav, right, extra, noShadow }) => {
  const headerFallback = useSingleNavbarBreadcrumb('Vault');

  return (
    <Navbar
      left={left}
      header={header ?? headerFallback}
      nav={nav}
      right={right ?? <DashboardUserIconDropdown />}
      extra={extra}
      noShadow={noShadow}
    />
  );
});
