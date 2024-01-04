import { memo, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

import { DashboardError } from '../components/dashboard/DashboardError';
import { DashboardLoading } from '../components/dashboard/DashboardLoading';
import { useSubscribe } from '../webVault/useSubscribe';
import { useWebVault } from '../webVault/useWebVault';

export const HomePage = memo(() => {
  const navigate = useNavigate();
  const webVault = useWebVault();
  const [repos] = useSubscribe(
    (v, cb) => v.reposSubscribe(cb),
    (v) => v.reposData,
    [],
  );

  useEffect(() => {
    if (repos?.status.type !== 'Loading' && repos?.status.type !== 'Error') {
      webVault.load();
    }

    if (repos?.status.type === 'Loaded') {
      if (repos.repos.length === 0) {
        navigate('/repos/create', { replace: true });
      } else {
        navigate(`/repos/${repos.repos[0].id}`, { replace: true });
      }
    }
  }, [webVault, repos, navigate]);

  if (repos?.status.type === 'Error') {
    return (
      <DashboardError
        error={repos.status.error}
        onRetry={() => {
          webVault.load();
        }}
      />
    );
  }

  return <DashboardLoading />;
});
