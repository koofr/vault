import { memo } from 'react';
import { useIntl } from 'react-intl';

import { ErrorComponent } from '../../components/ErrorComponent';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useDocumentTitle } from '../../utils/useDocumentTitle';

export const DashboardError = memo<{ error: string; onRetry?: () => void }>(
  ({ error, onRetry }) => {
    const intl = useIntl();
    useDocumentTitle(
      intl.formatMessage({
        id: 'web.dashboard_error.title',
        description: 'Document title shown when an error occurs.',
        defaultMessage: 'Error',
      }),
    );

    return (
      <DashboardLayout>
        <ErrorComponent error={error} onRetry={onRetry} />
      </DashboardLayout>
    );
  },
);
DashboardError.displayName = 'DashboardError';
