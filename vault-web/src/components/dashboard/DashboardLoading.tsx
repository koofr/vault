import { css } from '@emotion/css';
import { memo, ReactNode } from 'react';

import { useDocumentTitle } from '../../utils/useDocumentTitle';

import { LoadingCircle } from '../LoadingCircle';

import { DashboardNavbar } from './DashboardNavbar';

export const DashboardLoading = memo<{
  navbarHeader?: ReactNode;
}>(({ navbarHeader }) => {
  useDocumentTitle('Loading');

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
