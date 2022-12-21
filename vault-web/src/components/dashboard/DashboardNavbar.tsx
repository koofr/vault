import { memo, ReactNode } from 'react';

import { Navbar } from '../navbar/Navbar';

import { DashboardUserIconDropdown } from './DashboardUserIconDropdown';

export const DashboardNavbar = memo<{
  left?: ReactNode;
  header?: ReactNode;
  nav?: ReactNode;
  extra?: ReactNode;
}>(({ left, header, nav, extra }) => {
  return (
    <Navbar
      left={left}
      header={header ?? <span>Koofr Vault</span>}
      nav={nav}
      right={<DashboardUserIconDropdown />}
      extra={extra}
    />
  );
});
