import { css } from '@emotion/css';
import { memo, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';

import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useIsMobile } from '../../components/useIsMobile';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';

import { RepoFilesBreadcrumbs } from './RepoFilesBreadcrumbs';
import { RepoFilesBrowserIdContext } from './RepoFilesBrowserId';
import { RepoFilesContent } from './RepoFilesContent';
import { RepoFilesDropZoneComponent } from './RepoFilesDropZone';
import { RepoFilesMoveModal } from './RepoFilesMoveModal';
import { RepoFilesNav } from './RepoFilesNav';
import { RepoFilesNavbarExtra } from './RepoFilesNavbarExtra';
import { RepoFilesNavbarLeft } from './RepoFilesNavbarLeft';
import {
  RepoFilesUploadApi,
  RepoFilesUploadApiContext,
  RepoFilesUploadForm,
} from './RepoFilesUploadForm';
import { useBrowser } from './useBrowser';
import { useSelectName } from './useSelectName';

export const RepoFiles = memo<{ repo: Repo }>(({ repo }) => {
  const isMobile = useIsMobile();
  const repoId = repo.id;
  const [searchParams] = useSearchParams();
  const path = searchParams.get('path') ?? undefined;

  const selectName = useSelectName(repoId, path);

  const browserId = useBrowser(repoId, path ?? '/', selectName);

  const [info] = useSubscribe(
    (v, cb) => v.repoFilesBrowsersInfoSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersInfoData,
    [browserId]
  );

  const [breadcrumbs] = useSubscribe(
    (v, cb) => v.repoFilesBrowsersBreadcrumbsSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersBreadcrumbsData,
    [browserId]
  );
  const documentTitle = useMemo(
    () => breadcrumbs.map((bc) => bc.name).join(' › '),
    [breadcrumbs]
  );
  useDocumentTitle(documentTitle);

  const uploadApi: RepoFilesUploadApi = useMemo(() => ({}), []);

  return (
    <RepoFilesBrowserIdContext.Provider value={browserId}>
      <RepoFilesUploadApiContext.Provider value={uploadApi}>
        <DashboardLayout
          navbarLeft={<RepoFilesNavbarLeft />}
          navbarHeader={<RepoFilesBreadcrumbs />}
          navbarNav={<RepoFilesNav />}
          navbarExtra={<RepoFilesNavbarExtra info={info} />}
          className={
            isMobile
              ? css`
                  padding-top: 0;
                `
              : css`
                  padding-top: 10px;
                `
          }
          sidenavClassName={
            isMobile
              ? css``
              : css`
                  margin-top: -50px;
                `
          }
        >
          {info !== undefined ? <RepoFilesContent info={info} /> : null}

          <RepoFilesMoveModal />
          <RepoFilesDropZoneComponent />
          <RepoFilesUploadForm />
        </DashboardLayout>
      </RepoFilesUploadApiContext.Provider>
    </RepoFilesBrowserIdContext.Provider>
  );
});
