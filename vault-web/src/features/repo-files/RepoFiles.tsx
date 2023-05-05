import { css } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';

import {
  RenameFileModal,
  RenameFilePayload,
} from '../../components/RenameFileModal';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useIsMobile } from '../../components/useIsMobile';
import { useModal } from '../../utils/useModal';
import { Repo, RepoFile } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoFilesBreadcrumbs } from './RepoFilesBreadcrumbs';
import { RepoFilesBrowserIdContext } from './RepoFilesBrowserId';
import { RepoFilesContent } from './RepoFilesContent';
import { RepoFilesDropZoneComponent } from './RepoFilesDropZone';
import { RepoFilesMoveModal } from './RepoFilesMoveModal';
import { RepoFilesNav } from './RepoFilesNav';
import { RepoFilesNavbarExtra } from './RepoFilesNavbarExtra';
import { RepoFilesNavbarLeft } from './RepoFilesNavbarLeft';
import { RepoFilesRenameContext } from './RepoFilesRename';
import {
  RepoFilesUploadApi,
  RepoFilesUploadApiContext,
  RepoFilesUploadForm,
} from './RepoFilesUploadForm';

export const RepoFiles = memo<{ repo: Repo }>(({ repo }) => {
  const isMobile = useIsMobile();
  const repoId = repo.id;
  const [searchParams] = useSearchParams();
  const path = searchParams.get('path') ?? '/';
  const webVault = useWebVault();
  const browserId = useMemo(() => {
    return webVault.repoFilesBrowsersCreate(repoId, path);
    // we create a new browser with repoId and path and then use setLocation to update
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [webVault]);
  useEffect(() => {
    return () => {
      webVault.repoFilesBrowsersDestroy(browserId);
    };
  }, [webVault, browserId]);
  const setLocationCount = useRef(0);
  useEffect(() => {
    if (setLocationCount.current > 0) {
      webVault.repoFilesBrowsersSetLocation(browserId, repoId, path);
    }

    setLocationCount.current += 1;
  }, [webVault, browserId, repoId, path]);

  const [info] = useSubscribe(
    (v, cb) => v.repoFilesBrowsersInfoSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersInfoData,
    [browserId]
  );

  const uploadApi: RepoFilesUploadApi = useMemo(() => ({}), []);

  const renameFileModal = useModal<RenameFilePayload>();
  const renameFileModalShow = renameFileModal.show;
  const renameFile = useCallback(
    (file: RepoFile) => {
      const filePath = file.path;

      if (filePath !== undefined) {
        renameFileModalShow({
          originalName: file.name,
          isDir: file.type === 'Dir',
          canRenameFile: (name: string) =>
            webVault.repoFilesCanRenameFile(file.repoId, filePath, name),
          renameFile: (name: string) =>
            webVault.repoFilesRenameFile(file.repoId, filePath, name),
        });
      }
    },
    [webVault, renameFileModalShow]
  );

  return (
    <RepoFilesBrowserIdContext.Provider value={browserId}>
      <RepoFilesUploadApiContext.Provider value={uploadApi}>
        <RepoFilesRenameContext.Provider value={renameFile}>
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
                    z-index: 599;
                    padding-top: 10px;
                  `
            }
            sidenavClassName={
              isMobile
                ? css``
                : css`
                    margin-top: -50px;
                    z-index: 599;
                  `
            }
          >
            {info !== undefined ? <RepoFilesContent info={info} /> : null}

            <RepoFilesMoveModal />
            <RenameFileModal
              isVisible={renameFileModal.isVisible}
              payload={renameFileModal.payload}
              hide={renameFileModal.hide}
            />
            <RepoFilesDropZoneComponent />
            <RepoFilesUploadForm />
          </DashboardLayout>
        </RepoFilesRenameContext.Provider>
      </RepoFilesUploadApiContext.Provider>
    </RepoFilesBrowserIdContext.Provider>
  );
});
