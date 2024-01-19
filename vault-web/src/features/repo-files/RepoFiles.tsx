import { css } from '@emotion/css';
import { memo, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';

import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { DashboardLoading } from '../../components/dashboard/DashboardLoading';
import { useIsMobile } from '../../components/useIsMobile';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';

import { RepoGuard } from '../repo/RepoGuard';

import { RepoFileInfoModal } from './RepoFileInfoModal';
import { RepoFileInfoSheet } from './RepoFileInfoSheet';
import { RepoFilesBreadcrumbs } from './RepoFilesBreadcrumbs';
import { RepoFilesBrowserIdContext } from './RepoFilesBrowserId';
import { RepoFilesContent } from './RepoFilesContent';
import { RepoFilesDropZone } from './RepoFilesDropZone';
import { RepoFilesMoveModal } from './RepoFilesMoveModal';
import { RepoFilesNav } from './RepoFilesNav';
import { RepoFilesNavbarExtra } from './RepoFilesNavbarExtra';
import { RepoFilesNavbarLeft } from './RepoFilesNavbarLeft';
import {
  RepoFilesUploadApi,
  RepoFilesUploadApiContext,
} from './RepoFilesUploadApi';
import { RepoFilesUploadForm } from './RepoFilesUploadForm';
import { useBrowser } from './useBrowser';
import { useRepoFileInfo } from './useRepoFileInfo';
import { useSelectName } from './useSelectName';

export const RepoFilesInfo = memo<{
  browserId: number;
  info: RepoFilesBrowserInfo;
}>(({ browserId, info }) => {
  const isMobile = useIsMobile();

  const documentTitle = useMemo(
    () => info.breadcrumbs.map((bc) => bc.name).join(' › '),
    [info],
  );
  useDocumentTitle(documentTitle);

  const { onInfoClick, infoSheetVisible, infoSheetHide, infoModal } =
    useRepoFileInfo(info);

  const uploadApi: RepoFilesUploadApi = useMemo(() => ({}), []);

  return (
    <RepoFilesBrowserIdContext.Provider value={browserId}>
      <RepoFilesUploadApiContext.Provider value={uploadApi}>
        <DashboardLayout
          navbarLeft={
            info !== undefined ? (
              <RepoFilesNavbarLeft breadcrumbs={info.breadcrumbs} />
            ) : undefined
          }
          navbarHeader={
            info !== undefined ? (
              <RepoFilesBreadcrumbs breadcrumbs={info.breadcrumbs} />
            ) : undefined
          }
          navbarNav={<RepoFilesNav />}
          navbarExtra={
            <RepoFilesNavbarExtra info={info} onInfoClick={onInfoClick} />
          }
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
          <RepoFilesDropZone />
          <RepoFilesUploadForm />
          <RepoFileInfoSheet
            file={infoSheetVisible ? info?.selectedFile : undefined}
            hide={infoSheetHide}
          />
          <RepoFileInfoModal file={infoModal.payload} hide={infoModal.hide} />
        </DashboardLayout>
      </RepoFilesUploadApiContext.Provider>
    </RepoFilesBrowserIdContext.Provider>
  );
});

export const RepoFiles = memo<{ repoId: string }>(({ repoId }) => {
  const [searchParams] = useSearchParams();
  const path = searchParams.get('path') ?? undefined;

  const selectName = useSelectName(repoId, path);
  const browserId = useBrowser(repoId, path ?? '/', selectName);

  const [info] = useSubscribe(
    (v, cb) => v.repoFilesBrowsersInfoSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersInfoData,
    [browserId],
  );

  if (info === undefined) {
    return <DashboardLoading />;
  }

  return (
    <RepoGuard
      repoId={repoId}
      repoStatus={info.repoStatus}
      isLocked={info.isLocked}
    >
      <RepoFilesInfo browserId={browserId} info={info} />
    </RepoGuard>
  );
});
