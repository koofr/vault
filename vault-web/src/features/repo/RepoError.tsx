import { memo } from 'react';

import { ErrorComponent } from '../../components/ErrorComponent';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';

export const RepoError = memo<{ error: string }>(({ error }) => {
  return (
    <DashboardLayout>
      <ErrorComponent error={error} />
    </DashboardLayout>
  );
});
