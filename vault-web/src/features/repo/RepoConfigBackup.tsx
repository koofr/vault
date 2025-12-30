import { css, cx } from '@emotion/css';
import { memo, useMemo, useState } from 'react';

import { TextInput } from '../../components/TextInput';
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

export const RepoConfigBackupRepo = memo<{ repo: Repo }>(({ repo }) => {
  const repoId = repo.id;
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  const [password, setPassword] = useState('');
  const config = useMemo(
    () => webVault.reposGetRepoConfig(repoId, password),
    [webVault, repoId, password],
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
      {config !== undefined ? (
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

          <div
            className={css`
              margin: 0 0 40px;
            `}
          >
            <div
              className={css`
                margin: 0 0 10px;
              `}
            >
              To generate your rclone config, please type your Safe Key. Make
              sure it&apos;s correct.
            </div>
            <TextInput
              type="text"
              name="password"
              value={password}
              placeholder="Your Safe Key"
              onChange={(event) => setPassword(event.currentTarget.value)}
              className={cx(css`
                font-size: 16px;
                width: 250px;
                padding-right: 38px;
              `)}
              aria-label={'Safe Key'}
            />
          </div>

          <RepoConfigInfo config={config} />
        </div>
      ) : null}
    </DashboardLayout>
  );
});
RepoConfigBackupRepo.displayName = 'RepoConfigBackupRepo';

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
RepoConfigBackup.displayName = 'RepoConfigBackup';
