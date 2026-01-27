import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { format } from 'date-fns/format';
import { memo, useCallback } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button, LinkButton } from '../../components/Button';
import { DashboardError } from '../../components/dashboard/DashboardError';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useIsMobile } from '../../components/useIsMobile';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { useModal } from '../../utils/useModal';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { useDateFnsLocale } from '../intl/DateFnsLocaleContext';

import { RepoLock } from './RepoLock';
import { RepoRemoveModal } from './RepoRemoveModal';
import { RepoSpaceUsage } from './RepoSpaceUsage';

export const RepoInfoComponentRepo = memo<{ repo: Repo }>(({ repo }) => {
  const theme = useTheme();
  const dateFnsLocale = useDateFnsLocale();
  const isMobile = useIsMobile();
  const removeModal = useModal<Repo>();
  const removeModalShow = removeModal.show;
  const onRemove = useCallback(() => {
    removeModalShow(repo);
  }, [removeModalShow, repo]);
  const navbarHeader = useSingleNavbarBreadcrumb(repo.name);
  useDocumentTitle(repo.name);

  return (
    <DashboardLayout navbarHeader={navbarHeader}>
      <div
        className={
          isMobile
            ? css`
                padding: 0 15px;
              `
            : undefined
        }
      >
        <div>
          <h1
            className={css`
              display: inline;
              font-size: 32px;
              font-weight: normal;
              margin: 0;
            `}
          >
            {repo.name}
          </h1>
          <small
            className={css`
              font-size: 13px;
              font-weight: normal;
              color: ${theme.colors.textLight};
              margin-left: 10px;
              text-transform: uppercase;
            `}
          >
            {repo.state === 'Locked' ? (
              <FormattedMessage
                id="web.repo_info.locked.text"
                description="Status label shown next to the Safe Box name when it is locked."
                defaultMessage="Locked"
              />
            ) : (
              <FormattedMessage
                id="web.repo_info.unlocked.text"
                description="Status label shown next to the Safe Box name when it is unlocked."
                defaultMessage="Unlocked"
              />
            )}
          </small>
        </div>
        <p
          className={css`
            margin: 0 0 25px;
            font-size: 12px;
          `}
        >
          <FormattedMessage
            id="web.repo_info.created_at.text"
            description="Line showing the Safe Box creation date on the info page (e.g. 'Created Saturday, January 1, 2022 at 12:00:00')."
            defaultMessage="Created {created}"
            values={{
              created: format(repo.added, 'PPPPpp', { locale: dateFnsLocale }),
            }}
          />
        </p>
        <div
          className={
            isMobile
              ? css`
                  display: flex;
                  flex-direction: column;
                  margin-bottom: 40px;
                `
              : css`
                  display: flex;
                  flex-direction: row;
                  margin-bottom: 50px;
                `
          }
        >
          <Button
            variant="primary"
            href={repo.webUrl}
            target="_blank"
            rel="noreferrer"
            className={
              isMobile
                ? css`
                    width: 100%;
                    margin-bottom: 15px;
                  `
                : css`
                    width: 200px;
                    margin-right: 20px;
                  `
            }
          >
            <FormattedMessage
              id="web.repo_info.open_repo_in_remote.button"
              description="Button label to open the Safe Box in Koofr."
              defaultMessage="Open in Koofr"
            />
          </Button>
          <Button
            type="button"
            variant="destructive"
            className={
              isMobile
                ? css`
                    width: 100%;
                  `
                : css`
                    width: 200px;
                  `
            }
            onClick={onRemove}
          >
            <FormattedMessage
              id="web.repo_info.destroy_repo.button"
              description="Destructive button label to open the destroy Safe Box dialog."
              defaultMessage="Destroy Safe Box…"
            />
          </Button>
        </div>

        <div
          className={css`
            margin-bottom: 50px;
          `}
        >
          <RepoLock repo={repo} />
        </div>

        <div
          className={css`
            margin-bottom: 50px;
          `}
        >
          <RepoSpaceUsage repoId={repo.id} />
        </div>

        <div className={css``}>
          <h2
            className={css`
              font-size: 28px;
              font-weight: normal;
              margin: 0 0 20px;
            `}
          >
            <FormattedMessage
              id="web.repo_info.backup_config.heading"
              description="Section header for the Safe Box configuration backup area."
              defaultMessage="Backup config"
            />
          </h2>
          <LinkButton to={`/repos/${repo.id}/configbackup`} variant="primary">
            <FormattedMessage
              id="web.repo_info.backup_config.button"
              description="Button label to navigate to the config backup screen."
              defaultMessage="Backup the Safe Box config"
            />
          </LinkButton>
        </div>
      </div>

      <RepoRemoveModal
        repoId={removeModal.isVisible ? repo.id : undefined}
        hide={removeModal.hide}
      />
    </DashboardLayout>
  );
});
RepoInfoComponentRepo.displayName = 'RepoInfoComponentRepo';

export const RepoInfoComponent = memo<{ repoId: string }>(({ repoId }) => {
  const webVault = useWebVault();
  const [info] = useSubscribe(
    (v, cb) => v.reposRepoSubscribe(repoId, cb),
    (v) => v.reposRepoData,
    [repoId],
  );

  if (info?.status.type === 'Error') {
    return (
      <DashboardError
        error={info.status.error}
        onRetry={() => webVault.load()}
      />
    );
  } else if (info?.repo !== undefined) {
    return <RepoInfoComponentRepo repo={info.repo} />;
  } else {
    return <DashboardLoading />;
  }
});
RepoInfoComponent.displayName = 'RepoInfoComponent';
