import { memo, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

import { DashboardLoading } from '../components/dashboard/DashboardLoading';
import { useSubscribe } from '../webVault/useSubscribe';

export const HomePage = memo(() => {
  const navigate = useNavigate();
  const [repos] = useSubscribe(
    (v, cb) => v.reposSubscribe(cb),
    (v) => v.reposData,
    []
  );

  useEffect(() => {
    if (repos.status.type === 'Loaded') {
      if (repos.repos.length === 0) {
        navigate('/repos/create', { replace: true });
      } else {
        navigate(`/repos/${repos.repos[0].id}`, { replace: true });
      }
    }
  }, [repos, navigate]);

  return <DashboardLoading />;
});
