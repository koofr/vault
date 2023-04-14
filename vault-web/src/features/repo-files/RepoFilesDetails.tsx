import { css } from '@emotion/css';
import { memo, useEffect, useMemo, useRef, useState } from 'react';
import { useSearchParams } from 'react-router-dom';

import { useDocumentSize } from '../../components/DocumentSize';
import { ErrorComponent } from '../../components/ErrorComponent';
import { LoadingCircle } from '../../components/LoadingCircle';
import { DashboardNavbar } from '../../components/dashboard/DashboardNavbar';
import { getNavbarHeight } from '../../components/navbar/Navbar';
import { NavbarClose } from '../../components/navbar/NavbarClose';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useIsMobile } from '../../components/useIsMobile';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { fileHasPdfViewer, pdfViewerUrl, repoFilesLink } from './selectors';

const RepoFilesDetailsInner = memo<{ repo: Repo; path: string }>(
  ({ repo, path }) => {
    const isMobile = useIsMobile();
    const repoId = repo.id;
    const webVault = useWebVault();
    const detailsId = useMemo(() => {
      return webVault.repoFilesDetailsCreate(repoId, path);
    }, [webVault, repoId, path]);
    useEffect(() => {
      return () => {
        webVault.repoFilesDetailsDestroy(detailsId);
      };
    }, [webVault, detailsId]);
    const info = useSubscribe(
      (v, cb) => v.repoFilesDetailsInfoSubscribe(detailsId, cb),
      (v) => v.repoFilesDetailsInfoData,
      [detailsId]
    );
    const file = info?.file;
    const fileId = file?.id;
    const [blobUrl, setBlobUrl] = useState<string>();
    const lastBlobUrl = useRef<string>();
    useEffect(() => {
      const getFile = async () => {
        const stream = await webVault.repoFilesDetailsGetFileStream(
          detailsId,
          true
        );

        if (stream === undefined || stream.blob === undefined) {
          return;
        }

        var blobUrl = URL.createObjectURL(stream.blob);

        lastBlobUrl.current = blobUrl;
        setBlobUrl(blobUrl);
      };

      if (fileId !== undefined) {
        getFile();
      }
    }, [webVault, detailsId, fileId]);
    useEffect(() => {
      return () => {
        if (lastBlobUrl.current !== undefined) {
          URL.revokeObjectURL(lastBlobUrl.current);
          lastBlobUrl.current = undefined;
        }
      };
    }, [webVault]);
    const navbarHeader = useSingleNavbarBreadcrumb(file?.name ?? '');
    const documentSize = useDocumentSize();

    const viewerWidth = documentSize.width;
    const viewerHeight = documentSize.height - getNavbarHeight(isMobile);
    const viewerUrl =
      file !== undefined && fileHasPdfViewer(file) && blobUrl !== undefined
        ? pdfViewerUrl(blobUrl)
        : undefined;

    return (
      <>
        <DashboardNavbar
          header={navbarHeader}
          right={
            <NavbarClose
              to={
                info?.repoId !== undefined
                  ? repoFilesLink(info.repoId, info.parentPath ?? '/')
                  : '/'
              }
            />
          }
          noShadow={viewerUrl !== undefined}
        />

        {info?.status.type === 'Error' ? (
          <ErrorComponent error={info.status.error} />
        ) : viewerUrl !== undefined ? (
          <div>
            <iframe
              title="Viewer"
              src={viewerUrl}
              width={viewerWidth}
              height={viewerHeight}
              className={css`
                border: none;
                display: block;
              `}
            />
          </div>
        ) : info?.status.type === 'Loading' ||
          info?.status.type === 'Reloading' ? (
          <LoadingCircle />
        ) : null}
      </>
    );
  }
);

export const RepoFilesDetails = memo<{ repo: Repo }>(({ repo }) => {
  const [searchParams] = useSearchParams();
  const path = searchParams.get('path') ?? '/';
  const key = `${repo.id}:${path}`;

  // we specify the key so that the component is recreated when repo or path change
  return <RepoFilesDetailsInner key={key} repo={repo} path={path} />;
});
