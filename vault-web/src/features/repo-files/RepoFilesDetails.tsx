import { memo, useEffect, useMemo, useRef, useState } from 'react';
import { useSearchParams } from 'react-router-dom';

import {
  DocumentSizeInfo,
  useDocumentSize,
} from '../../components/DocumentSize';
import { ErrorComponent } from '../../components/ErrorComponent';
import { LoadingCircle } from '../../components/LoadingCircle';
import { PdfViewer } from '../../components/PdfViewer';
import { TextEditor } from '../../components/TextEditor';
import { DashboardNavbar } from '../../components/dashboard/DashboardNavbar';
import { getNavbarHeight } from '../../components/navbar/Navbar';
import { NavbarClose } from '../../components/navbar/NavbarClose';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { isDocumentSizeMobile } from '../../components/useIsMobile';
import { Repo, RepoFile } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import {
  fileHasPdfViewer,
  fileHasTextEditor,
  repoFilesLink,
} from './selectors';

const getContentEl = (
  file: RepoFile | undefined,
  blobUrl: string | undefined,
  documentSize: DocumentSizeInfo
): React.ReactElement | undefined => {
  if (file === undefined || blobUrl === undefined) {
    return undefined;
  }

  const isMobile = isDocumentSizeMobile(documentSize);
  const width = documentSize.width;
  const height = documentSize.height - getNavbarHeight(isMobile);

  if (fileHasPdfViewer(file)) {
    return <PdfViewer blobUrl={blobUrl} width={width} height={height} />;
  } else if (fileHasTextEditor(file)) {
    return (
      <TextEditor
        fileName={file.name}
        blobUrl={blobUrl}
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

    const contentEl = getContentEl(file, blobUrl, documentSize);

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
