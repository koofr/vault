import { css } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo } from 'react';

import { DashboardError } from '../../components/dashboard/DashboardError';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { NavbarBreadcrumbInfo } from '../../components/navbar/NavbarBreadcrumb';
import { NavbarBreadcrumbs } from '../../components/navbar/NavbarBreadcrumbs';
import { useIsMobile } from '../../components/useIsMobile';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoConfigInfo } from './RepoConfigInfo';
import { RepoUnlockForm } from './RepoUnlockForm';

export const RepoConfigBackupRepo = memo<{ repo: Repo }>(({ repo }) => {
  const repoId = repo.id;
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  let backupId = useMemo(
    () => webVault.repoConfigBackupCreate(repoId),
    [webVault, repoId],
  );
  useEffect(() => {
    return () => {
      webVault.repoConfigBackupDestroy(backupId);
    };
  }, [webVault, backupId]);
  const [info] = useSubscribe(
    (v, cb) => v.repoConfigBackupInfoSubscribe(backupId, cb),
    (v) => v.repoConfigBackupInfoData,
    [backupId],
  );
  const onUnlock = useCallback(
    (password: string) => {
      webVault.repoConfigBackupGenerate(backupId, password);
    },
    [webVault, backupId],
  );
  const breadcrumbs = useMemo(
    (): NavbarBreadcrumbInfo[] => [
      {
        id: repo.id,
        name: repo.name,
        link: `/repos/${repo.id}`,
        isClickable: true,
        hasCaret: false,
        isLast: false,
      },
      {
        id: 'configbackup',
        name: 'Backup config',
        isClickable: false,
        hasCaret: false,
        isLast: true,
      },
    ],
    [repo],
  );
  useDocumentTitle('Backup config');

  return (
    <DashboardLayout
      navbarHeader={<NavbarBreadcrumbs breadcrumbs={breadcrumbs} />}
    >
      {info !== undefined ? (
        info.config === undefined ? (
          <RepoUnlockForm info={info.unlockInfo} onUnlock={onUnlock} />
        ) : (
          <div
            className={
              isMobile
                ? css`
                    padding: 0 15px;
                  `
                : undefined
            }
          >
            <h1
              className={css`
                font-size: 32px;
                font-weight: normal;
                margin: 0 0 20px;
              `}
            >
              Backup config
            </h1>

            <RepoConfigInfo config={info.config} />
          </div>
        )
      ) : null}
    </DashboardLayout>
  );
});

export const RepoConfigBackup = memo<{ repoId: string }>(({ repoId }) => {
  const [info] = useSubscribe(
    (v, cb) => v.reposRepoSubscribe(repoId, cb),
    (v) => v.reposRepoData,
    [repoId],
  );

  if (info?.status.type === 'Error') {
    return <DashboardError error={info.status.error} />;
  } else if (info?.repo !== undefined) {
    return <RepoConfigBackupRepo repo={info.repo} />;
  } else {
    return <DashboardLoading />;
  }
});
