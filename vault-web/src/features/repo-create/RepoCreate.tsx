import { memo, useEffect, useMemo } from 'react';

import { LoadingCircle } from '../../components/LoadingCircle';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoCreateCreatedComponent } from './RepoCreateCreatedComponent';
import { RepoCreateFormComponent } from './RepoCreateFormComponent';

export const RepoCreate = memo(() => {
  const webVault = useWebVault();
  const createId = useMemo(() => webVault.repoCreateCreate(), [webVault]);
  const [info] = useSubscribe(
    (v, cb) => v.repoCreateInfoSubscribe(createId, cb),
    (v) => v.repoCreateInfoData,
    [],
  );
  useEffect(() => {
    return () => {
      webVault.repoCreateDestroy(createId);
    };
  }, [webVault, createId]);
  const navbarHeader = useSingleNavbarBreadcrumb('Create a new Safe Box');
  useDocumentTitle('Create a new Safe Box');

  return (
    <DashboardLayout navbarHeader={navbarHeader}>
      {info?.type === 'Form' ? (
        <RepoCreateFormComponent createId={createId} form={info} />
      ) : info?.type === 'Created' ? (
        <RepoCreateCreatedComponent created={info} />
      ) : (
        <LoadingCircle />
      )}
    </DashboardLayout>
  );
});
