import { css } from '@emotion/css';
import format from 'date-fns/format';
import { memo, useCallback } from 'react';
import { Link } from 'react-router-dom';

import { Button, LinkButton } from '../../components/Button';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useIsMobile } from '../../components/useIsMobile';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { useModal } from '../../utils/useModal';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';

import { RepoRemoveModal } from './RepoRemoveModal';
import { RepoError } from './RepoError';
import { RepoSpaceUsage } from './RepoSpaceUsage';

export const RepoInfoRepo = memo<{ repo: Repo }>(({ repo }) => {
  const isMobile = useIsMobile();
  const removeModal = useModal<Repo>();
  const removeModalShow = removeModal.show;
  const onRemove = useCallback(async () => {
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
        <h1
          className={css`
            font-size: 32px;
            font-weight: normal;
            margin: 0;
          `}
        >
          {repo.name}
        </h1>
        <p
          className={css`
            margin: 0 0 25px;
            font-size: 12px;
          `}
        >
          Created {format(repo.added, 'PPPPpp')}
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
            Open in Koofr
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
            Destroy Safe Box...
          </Button>
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
            Backup config
          </h2>
          <LinkButton to={`/repos/${repo.id}/configbackup`} variant="primary">
            Backup the Safe Box config
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

export const RepoInfo = memo<{ repoId: string }>(({ repoId }) => {
  const [info] = useSubscribe(
    (v, cb) => v.reposRepoSubscribe(repoId, cb),
    (v) => v.reposRepoData,
    [repoId]
  );

  if (info.status.type === 'Error') {
    return <RepoError error={info.status.error} />;
  } else if (info.repo !== undefined) {
    return <RepoInfoRepo repo={info.repo} />;
  } else {
    return <DashboardLoading />;
  }
});
