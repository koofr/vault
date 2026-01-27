import { css } from '@emotion/css';
import { memo, ReactNode } from 'react';
import { useIntl } from 'react-intl';

import { useDocumentTitle } from '../../utils/useDocumentTitle';

import { LoadingCircle } from '../LoadingCircle';

import { DashboardNavbar } from './DashboardNavbar';

export const DashboardLoading = memo<{
  navbarHeader?: ReactNode;
}>(({ navbarHeader }) => {
  const intl = useIntl();
  useDocumentTitle(
    intl.formatMessage({
      id: 'web.dashboard_loading.title',
      description: 'Document title shown while the dashboard is loading.',
      defaultMessage: 'Loading',
    }),
  );

  return (
    <>
      <DashboardNavbar header={navbarHeader ?? ''} />

      <main
        className={css`
          display: flex;
          flex-direction: column;
          flex-grow: 1;
        `}
      >
        <LoadingCircle />
      </main>
    </>
  );
});
DashboardLoading.displayName = 'DashboardLoading';
