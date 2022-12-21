import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

import { DashboardLoading } from '../components/dashboard/DashboardLoading';
import { useWebVault } from '../webVault/useWebVault';

import { getLoginRedirect, removeLoginRedirect } from './loginRedirect';

export const OAuth2CallbackPage: React.FC<{}> = () => {
  const webVault = useWebVault();
  const navigate = useNavigate();
  const locationHref = document.location.href;

  useEffect(() => {
    (async () => {
      const success = await webVault.oauth2FinishFlowUrl(locationHref);

      if (success) {
        const loginRedirect = getLoginRedirect();
        removeLoginRedirect();

        navigate(loginRedirect ?? '/', {
          replace: true,
        });
      } else {
        navigate('/', {
          replace: true,
        });
      }
    })();
  }, [webVault, locationHref, navigate]);

  return <DashboardLoading />;
};
