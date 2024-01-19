import { memo, useContext, useEffect } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';

import { DashboardLoading } from '../components/dashboard/DashboardLoading';
import { LandingPageContext } from '../landingPageContext';
import { useSubscribe } from '../webVault/useSubscribe';

import { setLoginRedirect } from './loginRedirect';

export const AuthGuard = memo(() => {
  const landingPage = useContext(LandingPageContext);
  const [oauth2Status] = useSubscribe(
    (v, cb) => v.oauth2StatusSubscribe(cb),
    (v) => v.oauth2StatusData,
    [],
  );

  const navigate = useNavigate();
  const location = useLocation();
  const locationRelUrl = location.pathname + location.search;
  const needsRedirectToLogin =
    import.meta.env.VITE_VAULT_APP === 'desktop'
      ? false
      : locationRelUrl !== '/' &&
        oauth2Status?.type !== 'Loading' &&
        oauth2Status?.type !== 'Loaded';

  useEffect(() => {
    if (needsRedirectToLogin) {
      // remember the current url so that the user is redirected to it after
      // a successful login
      setLoginRedirect(locationRelUrl);

      navigate('/login', {
        replace: true,
      });
    }
  }, [needsRedirectToLogin, locationRelUrl, navigate]);

  if (needsRedirectToLogin) {
    return null;
  } else if (oauth2Status?.type === 'Loading') {
    return <DashboardLoading />;
  } else if (oauth2Status?.type === 'Loaded') {
    return <Outlet />;
  } else {
    return landingPage;
  }
});
