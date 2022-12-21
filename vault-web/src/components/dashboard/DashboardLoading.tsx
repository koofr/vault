import { memo, ReactNode } from 'react';

import { LoadingCircle } from '../LoadingCircle';

import { DashboardNavbar } from './DashboardNavbar';

export const DashboardLoading = memo<{
  navbarHeader?: ReactNode;
}>(({ navbarHeader }) => {
  return (
    <>
      <DashboardNavbar header={navbarHeader ?? ''} />

      <LoadingCircle />
    </>
  );
});
