import { css } from '@emotion/css';
import { MutableRefObject, memo, useEffect, useRef, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { useDocumentSize } from '../../components/DocumentSize';
import { ErrorComponent } from '../../components/ErrorComponent';
import { LoadingCircle } from '../../components/LoadingCircle';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { usePreventUnload } from '../../utils/usePreventUnload';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { repoFilesDetailsLink, repoFilesLink } from '../repo-files/selectors';
import { Transfers } from '../transfers/Transfers';

import { getContentEl } from './RepoFilesDetailsContent';
import { RepoFilesDetailsNavbar } from './RepoFilesDetailsNavbar';
import { useDetails } from './useDetails';
import { useShortcuts } from './useShortcuts';

const RepoFilesDetailsInner = memo<{
  repoId: string;
  encryptedPath: string;
  isEditing: boolean;
  autosaveIntervalMs?: number;
  expectedEncryptedNewPath: MutableRefObject<string | undefined>;
}>(
  ({
    repoId,
    encryptedPath,
    isEditing,
    autosaveIntervalMs,
    expectedEncryptedNewPath,
  }) => {
    const webVault = useWebVault();
    const navigate = useNavigate();
    const detailsId = useDetails(
      repoId,
      encryptedPath,
      isEditing,
      autosaveIntervalMs,
    );

    const [info, infoRef] = useSubscribe(
      (v, cb) => v.repoFilesDetailsInfoSubscribe(detailsId, cb),
      (v) => v.repoFilesDetailsInfoData,
      [detailsId],
    );

    useDocumentTitle(info?.fileName);

    useEffect(() => {
      if (
        info !== undefined &&
        info.repoId !== undefined &&
        info.encryptedPath !== undefined
      ) {
        if (info.shouldDestroy) {
          // TODO navigate to parent and select the file
          navigate(
            repoFilesLink(
              info.repoId,
              info.encryptedParentPath ?? '/',
              info.fileName,
            ),
          );
        } else if (info.encryptedPath !== encryptedPath) {
          expectedEncryptedNewPath.current = info.encryptedPath;

          navigate(
            repoFilesDetailsLink(
              info.repoId,
              info.encryptedPath,
              info.isEditing,
              autosaveIntervalMs,
            ),
            { replace: true },
          );
        }
      }
    }, [
      info,
      encryptedPath,
      autosaveIntervalMs,
      navigate,
      expectedEncryptedNewPath,
    ]);

    useShortcuts(detailsId, infoRef);

    usePreventUnload(info?.isDirty ?? false);

    const documentSize = useDocumentSize();

    if (info === undefined) {
      return null;
    }

    const contentEl = getContentEl(
      detailsId,
      info.fileName,
      info.fileExt,
      info.fileCategory,
      info.contentStatus,
      info.isEditing,
      documentSize,
    );

    return (
      <>
        <RepoFilesDetailsNavbar detailsId={detailsId} info={info} />

        <main
          className={css`
            display: flex;
            flex-direction: column;
            flex-grow: 1;
          `}
        >
          {info.status.type === 'Error' && !info.isDirty ? (
            <ErrorComponent
              error={info.status.error}
              onRetry={() => {
                webVault.repoFilesDetailsLoadFile(detailsId);
              }}
            />
          ) : contentEl !== undefined ? (
            contentEl
          ) : info.status.type === 'Loading' ? (
            <LoadingCircle />
          ) : null}
        </main>

        <Transfers />
      </>
    );
  },
);

function getAutosaveIntervalMs(
  searchParams: URLSearchParams,
): number | undefined {
  const raw = searchParams.get('autosave');
  if (raw === null || raw === '') {
    return undefined;
  }
  const parsed = parseInt(raw, 10);
  if (Number.isNaN(parsed)) {
    return undefined;
  }
  return parsed;
}

export const RepoFilesDetails = memo<{ repoId: string }>(({ repoId }) => {
  const [searchParams] = useSearchParams();
  const encryptedPath = searchParams.get('path') ?? '/';
  const isEditing = searchParams.get('editing') === 'true';
  let autosaveIntervalMs = getAutosaveIntervalMs(searchParams);

  const [currentEncryptedPath, setCurrentEncryptedPath] =
    useState(encryptedPath);
  const expectedEncryptedNewPath = useRef<string>();
  const [key, setKey] = useState(0);

  if (encryptedPath !== currentEncryptedPath) {
    setCurrentEncryptedPath(encryptedPath);
    if (encryptedPath !== expectedEncryptedNewPath.current) {
      setKey((key) => key + 1);
    }
    expectedEncryptedNewPath.current = undefined;
  }

  // we specify the key so that the component is recreated when repo or encryptedPath
  // change
  return (
    <RepoFilesDetailsInner
      key={key}
      repoId={repoId}
      encryptedPath={encryptedPath}
      isEditing={isEditing}
      autosaveIntervalMs={autosaveIntervalMs}
      expectedEncryptedNewPath={expectedEncryptedNewPath}
    />
  );
});
