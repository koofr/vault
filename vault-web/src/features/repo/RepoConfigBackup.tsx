import { css, cx } from '@emotion/css';
import { memo, useMemo, useState } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

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
  const intl = useIntl();
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  const [password, setPassword] = useState('');
  const config = useMemo(
    () => webVault.reposGetRepoConfig(repoId, password),
    [webVault, repoId, password],
  );
  const title = intl.formatMessage({
    id: 'web.repo_config_backup.title',
    description: 'Document title for the Safe Box configuration backup page.',
    defaultMessage: 'Backup config',
  });
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
        name: title,
        isClickable: false,
        hasCaret: false,
        isLast: true,
      },
    ],
    [repo, title],
  );
  useDocumentTitle(title);

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
            {title}
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
              <FormattedMessage
                id="web.repo_config_backup.description"
                description="Instruction text explaining that the Safe Key is required to generate the rclone config."
                defaultMessage="To generate your rclone config, please type your Safe Key. Make sure it's correct."
              />
            </div>
            <TextInput
              type="text"
              name="password"
              value={password}
              placeholder={intl.formatMessage({
                id: 'web.repo_config_backup.password.placeholder',
                description:
                  'Placeholder text for the Safe Key input on the config backup page.',
                defaultMessage: 'Your Safe Key',
              })}
              onChange={(event) => setPassword(event.currentTarget.value)}
              className={cx(css`
                font-size: 16px;
                width: 250px;
                padding-right: 38px;
              `)}
              aria-label={intl.formatMessage({
                id: 'web.repo_config_backup.password.aria_label',
                description:
                  'Accessibility label for the Safe Key input on the config backup page.',
                defaultMessage: 'Safe Key',
              })}
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
