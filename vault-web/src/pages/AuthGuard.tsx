import { memo } from 'react';
import { Outlet } from 'react-router-dom';

import { DashboardLoading } from '../components/dashboard/DashboardLoading';
import { useSubscribe } from '../webVault/useSubscribe';

import { LandingPageLazy } from './LandingPageLazy';

export const AuthGuard = memo(() => {
  const oauth2Status = useSubscribe(
    (v, cb) => v.oauth2StatusSubscribe(cb),
    (v) => v.oauth2StatusData,
    []
  );

  if (oauth2Status.type === 'Loading') {
    return <DashboardLoading />;
  } else if (oauth2Status.type === 'Loaded') {
    return <Outlet />;
  } else {
    return <LandingPageLazy />;
  }
});
