import { useEffect, useMemo, useRef } from 'react';

import { useWebVault } from '../../../webVault/useWebVault';

export function useDetails(
  repoId: string,
  path: string,
  isEditing: boolean,
  autosaveIntervalMs?: number
): number {
  const webVault = useWebVault();

  const detailsId = useMemo(
    () => {
      // we create a new details with repoId, path and then use edit and
      // cancelEditing to control isEditing
      return webVault.repoFilesDetailsCreate(repoId, path, isEditing, {
        loadContent: {
          categories: ['Text', 'Code'],
          exts: [],
        },
        autosaveIntervalMs: autosaveIntervalMs ?? 20000,
      });
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [
      webVault,
      // repoId, path and autosaveIntervalMs will never change, isEditing must
      // not be a dependency
    ]
  );
  useEffect(() => {
    return () => {
      webVault.repoFilesDetailsDestroy(detailsId);
    };
  }, [webVault, detailsId]);
  const editingCount = useRef(0);
  useEffect(() => {
    if (editingCount.current > 0) {
      if (isEditing) {
        webVault.repoFilesDetailsEdit(detailsId);
      } else {
        webVault.repoFilesDetailsEditCancel(detailsId);
      }
    }

    editingCount.current += 1;
  }, [webVault, detailsId, isEditing]);

  return detailsId;
}
