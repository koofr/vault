import { css } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';

import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useIsMobile } from '../../components/useIsMobile';
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
  const browserId = useMemo(
    () => {
      // we create a new browser with repoId and path and then use setLocation
      // to update path
      return webVault.repoFilesBrowsersCreate(repoId, path);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [webVault]
  );
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
