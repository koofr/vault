import { css } from '@emotion/css';
import { MutableRefObject, memo, useEffect, useRef, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { useDocumentSize } from '../../../components/DocumentSize';
import { ErrorComponent } from '../../../components/ErrorComponent';
import { LoadingCircle } from '../../../components/LoadingCircle';
import { usePreventUnload } from '../../../utils/usePreventUnload';
import { Repo } from '../../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../../webVault/useSubscribe';

import { repoFilesDetailsLink, repoFilesLink } from '../selectors';

import { getContentEl } from './RepoFilesDetailsContent';
import { RepoFilesDetailsNavbar } from './RepoFilesDetailsNavbar';
import { useDetails } from './useDetails';
import { useShortcuts } from './useShortcuts';

const RepoFilesDetailsInner = memo<{
  repo: Repo;
  path: string;
  isEditing: boolean;
  autosaveIntervalMs?: number;
  expectedNewPath: MutableRefObject<string | undefined>;
}>(({ repo, path, isEditing, autosaveIntervalMs, expectedNewPath }) => {
  const navigate = useNavigate();
  const detailsId = useDetails(repo.id, path, isEditing, autosaveIntervalMs);

  const [info, infoRef] = useSubscribe(
    (v, cb) => v.repoFilesDetailsInfoSubscribe(detailsId, cb),
    (v) => v.repoFilesDetailsInfoData,
    [detailsId]
  );

  useEffect(() => {
    if (
      info !== undefined &&
      info.repoId !== undefined &&
      info.path !== undefined
    ) {
      if (info.shouldDestroy) {
        // TODO navigate to parent and select the file
        navigate(
          repoFilesLink(info.repoId, info.parentPath ?? '/', info.fileName)
        );
      } else if (info.path !== path) {
        expectedNewPath.current = info.path;

        navigate(
          repoFilesDetailsLink(
            info.repoId,
            info.path,
            info.isEditing,
            autosaveIntervalMs
          ),
          { replace: true }
        );
      }
    }
  }, [info, path, autosaveIntervalMs, navigate, expectedNewPath]);

  useShortcuts(detailsId, infoRef);

  usePreventUnload(info?.isDirty ?? false);

  const documentSize = useDocumentSize();

  if (info === undefined) {
    return null;
  }

  // const file = info?.file;

  const contentEl = getContentEl(
    detailsId,
    info.fileName,
    info.fileExt,
    info.fileCategory,
    info.contentStatus,
    info.isEditing,
    documentSize
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
          <ErrorComponent error={info.status.error} />
        ) : contentEl !== undefined ? (
          contentEl
        ) : info.status.type === 'Loading' ||
          info.status.type === 'Reloading' ? (
          <LoadingCircle />
        ) : null}
      </main>
    </>
  );
});

function getAutosaveIntervalMs(
  searchParams: URLSearchParams
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

export const RepoFilesDetails = memo<{ repo: Repo }>(({ repo }) => {
  const [searchParams] = useSearchParams();
  const path = searchParams.get('path') ?? '/';
  const isEditing = searchParams.get('editing') === 'true';
  let autosaveIntervalMs = getAutosaveIntervalMs(searchParams);

  const [currentPath, setCurrentPath] = useState(path);
  const expectedNewPath = useRef<string>();
  const [key, setKey] = useState(0);

  if (path !== currentPath) {
    setCurrentPath(path);
    if (path !== expectedNewPath.current) {
      setKey((key) => key + 1);
    }
    expectedNewPath.current = undefined;
  }

  // we specify the key so that the component is recreated when repo or path
  // change
  return (
    <RepoFilesDetailsInner
      key={key}
      repo={repo}
      path={path}
      isEditing={isEditing}
      autosaveIntervalMs={autosaveIntervalMs}
      expectedNewPath={expectedNewPath}
    />
  );
});
