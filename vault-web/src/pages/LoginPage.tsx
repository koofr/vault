import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

import { DashboardLoading } from '../components/dashboard/DashboardLoading';
import { useSubscribe } from '../webVault/useSubscribe';
import { useWebVault } from '../webVault/useWebVault';

export const LoginPage: React.FC = () => {
  const webVault = useWebVault();
  const navigate = useNavigate();
  const [oauth2Status] = useSubscribe(
    (v, cb) => v.oauth2StatusSubscribe(cb),
    (v) => v.oauth2StatusData,
    [],
  );

  useEffect(() => {
    if (oauth2Status?.type === 'Loaded') {
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      navigate('/', {
        replace: true,
      });
    } else {
      const url = webVault.oauth2StartLoginFlow();

      if (url !== undefined) {
        document.location.href = url;
      }
    }
  }, [webVault, oauth2Status, navigate]);

  return <DashboardLoading />;
};
LoginPage.displayName = 'LoginPage';
