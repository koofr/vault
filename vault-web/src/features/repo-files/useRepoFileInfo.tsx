import { useCallback, useState } from 'react';

import { useIsMobile } from '../../components/useIsMobile';
import { UseModal, useModal } from '../../utils/useModal';
import { RepoFile, RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';

const repoFileInfoSheetVisibleKey = 'vaultRepoFileInfoSheetVisible';

function localStorageGet(): boolean {
  try {
    if (localStorage.getItem(repoFileInfoSheetVisibleKey) === 'true') {
      return true;
    }
  } catch {}

  return false;
}

function localStorageSet(visible: boolean) {
  try {
    if (visible) {
      localStorage.setItem(repoFileInfoSheetVisibleKey, 'true');
    } else {
      localStorage.removeItem(repoFileInfoSheetVisibleKey);
    }
  } catch {}
}

export function useRepoFileInfo(info: RepoFilesBrowserInfo | undefined): {
  onInfoClick: () => void;
  infoSheetVisible: boolean;
  infoSheetHide: () => void;
  infoModal: UseModal<RepoFile>;
} {
  const isMobile = useIsMobile();

  const [infoSheetVisibleOriginal, setInfoSheetVisible] =
    useState(localStorageGet);
  const infoSheetVisible = infoSheetVisibleOriginal && !isMobile;

  const infoModal = useModal<RepoFile>();
  const infoModalShow = infoModal.show;

  const selectedFile = info?.selectedFile;

  const onInfoClick = useCallback(() => {
    if (isMobile) {
      if (selectedFile !== undefined) {
        infoModalShow(selectedFile);
      }
    } else {
      setInfoSheetVisible((value) => {
        const newValue = !value;

        localStorageSet(newValue);

        return newValue;
      });
    }
  }, [isMobile, infoModalShow, selectedFile]);

  const infoSheetHide = useCallback(() => {
    setInfoSheetVisible(false);

    localStorageSet(false);
  }, []);

  return { onInfoClick, infoSheetVisible, infoSheetHide, infoModal };
}
