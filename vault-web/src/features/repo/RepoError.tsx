import { memo } from 'react';

import { ErrorComponent } from '../../components/ErrorComponent';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useDocumentTitle } from '../../utils/useDocumentTitle';

export const RepoError = memo<{ error: string; onRetry?: () => void }>(
  ({ error, onRetry }) => {
    useDocumentTitle('Error');

    return (
      <DashboardLayout>
        <ErrorComponent error={error} onRetry={onRetry} />
      </DashboardLayout>
    );
  }
);
