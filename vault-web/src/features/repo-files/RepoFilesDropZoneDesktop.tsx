import { appWindow } from '@tauri-apps/api/window';
import { memo, useEffect, useState } from 'react';

import { DropZone } from '../../components/dnd/DropZone';
import { useWebVaultDesktop } from '../../desktopVault/useWebVaultDesktop';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';

export const RepoFilesDropZoneDesktop = memo(() => {
  const webVault = useWebVaultDesktop();
  const browserId = useRepoFilesBrowserId();

  const canUpload = true;

  const [isOver, setIsOver] = useState(false);
  const canDrop = isOver;

  useEffect(() => {
    const unlistenPromise = appWindow.onFileDropEvent((event) => {
      if (event.payload.type === 'hover') {
        setIsOver(true);
      } else if (event.payload.type === 'drop') {
        setIsOver(false);

        const { repoId, encryptedPath } =
          webVault.repoFilesBrowsersInfo(browserId)!;

        if (repoId !== undefined && encryptedPath !== undefined) {
          webVault.repoFilesUploadPaths(
            repoId,
            encryptedPath,
            event.payload.paths,
          );
        }
      } else {
        setIsOver(false);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });

  return <DropZone isActive={canDrop} isOver={isOver} isAllowed={canUpload} />;
});
