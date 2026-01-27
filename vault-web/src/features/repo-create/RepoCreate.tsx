import { memo, useEffect, useMemo } from 'react';
import { useIntl } from 'react-intl';

import { LoadingCircle } from '../../components/LoadingCircle';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoCreateCreatedComponent } from './RepoCreateCreatedComponent';
import { RepoCreateFormComponent } from './RepoCreateFormComponent';

export const RepoCreate = memo(() => {
  const intl = useIntl();
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
  const title = intl.formatMessage({
    id: 'web.repo_create.title',
    description: 'Title for the Safe Box creation page.',
    defaultMessage: 'Create a new Safe Box',
  });
  const navbarHeader = useSingleNavbarBreadcrumb(title);
  useDocumentTitle(title);

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
RepoCreate.displayName = 'RepoCreate';
