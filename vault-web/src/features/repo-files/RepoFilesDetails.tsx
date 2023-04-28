import { css } from '@emotion/css';
import { memo, useEffect, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';

import {
  DocumentSizeInfo,
  useDocumentSize,
} from '../../components/DocumentSize';
import { ErrorComponent } from '../../components/ErrorComponent';
import { ImageViewer } from '../../components/ImageViewer';
import { LoadingCircle } from '../../components/LoadingCircle';
import { PdfViewer } from '../../components/PdfViewer';
import { TextEditor } from '../../components/TextEditor';
import { DashboardNavbar } from '../../components/dashboard/DashboardNavbar';
import { getNavbarHeight } from '../../components/navbar/Navbar';
import { NavbarClose } from '../../components/navbar/NavbarClose';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { isDocumentSizeMobile } from '../../components/useIsMobile';
import { Repo, RepoFile, Status } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import {
  fileHasImageViewer,
  fileHasPdfViewer,
  fileHasTextEditor,
  repoFilesLink,
} from './selectors';
import { useRepoFilesDetailsBlobUrl } from './useRepoFilesDetailsBlobUrl';
import { useRepoFilesDetailsString } from './useRepoFilesDetailsBytes';

export const RepoFilesDetailsPdfViewer = memo<{
  detailsId: number;
  file: RepoFile;
  width: number;
  height: number;
}>(({ detailsId, file, width, height }) => {
  const blobUrl = useRepoFilesDetailsBlobUrl(detailsId, file);

  return blobUrl !== undefined ? (
    <PdfViewer url={blobUrl} width={width} height={height} />
  ) : (
    <LoadingCircle />
  );
});

export const RepoFilesDetailsTextEditor = memo<{
  detailsId: number;
  file: RepoFile;
  contentStatus: Status | undefined;
  width: number;
  height: number;
}>(({ detailsId, file, contentStatus, width, height }) => {
  const text = useRepoFilesDetailsString(detailsId, file);

  return contentStatus === undefined ||
    contentStatus.type === 'Loading' ||
    text === undefined ? (
    <LoadingCircle />
  ) : (
    <TextEditor
      fileName={file.name}
      text={text}
      width={width}
      height={height}
    />
  );
});

export const RepoFilesDetailsImageViewer = memo<{
  detailsId: number;
  file: RepoFile;
  contentStatus: Status | undefined;
  width: number;
  height: number;
}>(({ detailsId, file, contentStatus, width, height }) => {
  const blobUrl = useRepoFilesDetailsBlobUrl(detailsId, file);

  return contentStatus === undefined ||
    contentStatus.type === 'Loading' ||
    blobUrl === undefined ? (
    <LoadingCircle />
  ) : (
    <ImageViewer
      fileName={file.name}
      blobUrl={blobUrl}
      width={width}
      height={height}
    />
  );
});

const getContentEl = (
  detailsId: number,
  file: RepoFile | undefined,
  contentStatus: Status | undefined,
  documentSize: DocumentSizeInfo
): React.ReactElement | undefined => {
  if (file === undefined) {
    return undefined;
  }

  const isMobile = isDocumentSizeMobile(documentSize);
  const width = documentSize.width;
  const height = documentSize.height - getNavbarHeight(isMobile);

  if (fileHasPdfViewer(file)) {
    return (
      <RepoFilesDetailsPdfViewer
        detailsId={detailsId}
        file={file}
        width={width}
        height={height}
      />
    );
  } else if (fileHasTextEditor(file)) {
    return (
      <RepoFilesDetailsTextEditor
        detailsId={detailsId}
        file={file}
        contentStatus={contentStatus}
        width={width}
        height={height}
      />
    );
  } else if (fileHasImageViewer(file)) {
    return (
      <RepoFilesDetailsImageViewer
        detailsId={detailsId}
        file={file}
        contentStatus={contentStatus}
        width={width}
        height={height}
      />
    );
  }

  return undefined;
};

const RepoFilesDetailsInner = memo<{ repo: Repo; path: string }>(
  ({ repo, path }) => {
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
    const navbarHeader = useSingleNavbarBreadcrumb(file?.name ?? '');
    const documentSize = useDocumentSize();

    const contentEl = getContentEl(
      detailsId,
      file,
      info?.contentStatus,
      documentSize
    );

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
          noShadow={contentEl !== undefined}
        />

        <main
          className={css`
            display: flex;
            flex-direction: column;
            flex-grow: 1;
          `}
        >
          {info?.status.type === 'Error' ? (
            <ErrorComponent error={info.status.error} />
          ) : contentEl !== undefined ? (
            contentEl
          ) : info?.status.type === 'Loading' ||
            info?.status.type === 'Reloading' ? (
            <LoadingCircle />
          ) : null}
        </main>
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
