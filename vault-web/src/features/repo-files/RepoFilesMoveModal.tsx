import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback } from 'react';

import { Button } from '../../components/Button';
import { CreateDirModal } from '../../components/CreateDirModal';
import {
  Modal,
  ModalBody,
  ModalFooter,
  ModalFooterButton,
  ModalFooterButtons,
  ModalFooterExtra,
  ModalHeader,
  ModalTitle,
} from '../../components/modal/Modal';
import { useIsMobile } from '../../components/useIsMobile';
import { useModal } from '../../utils/useModal';
import { RepoFile, RepoFilesMoveInfo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoFilesDirPicker } from './RepoFilesDirPicker';

export interface RepoFileWithPath extends RepoFile {
  path: string;
}

export const RepoFilesMoveModalContent = memo<{
  info: RepoFilesMoveInfo;
  cancel: () => void;
}>(
  ({
    info: {
      srcFilesCount,
      mode,
      dirPickerId,
      destFileName,
      canMove,
      canShowCreateDir,
    },
    cancel,
  }) => {
    const isMobile = useIsMobile();
    const theme = useTheme();
    const webVault = useWebVault();
    const createDirModal = useModal();
    const canCreateDir = useCallback(
      (name: string) => webVault.repoFilesMoveCanCreateDir(name),
      [webVault]
    );
    const createDir = useCallback(
      (name: string) => {
        webVault.repoFilesMoveCreateDir(name);
      },
      [webVault]
    );
    const move = useCallback(async () => {
      webVault.repoFilesMoveMoveFiles();
    }, [webVault]);

    return (
      <>
        <ModalHeader>
          <ModalTitle
            className={css`
              font-weight: normal;
            `}
          >
            {mode === 'Copy' ? 'Copy' : 'Move'}{' '}
            <strong
              className={css`
                font-weight: 600;
              `}
            >
              {srcFilesCount} {srcFilesCount === 1 ? 'item' : 'items'}
            </strong>
            {destFileName !== undefined ? (
              <>
                {' to '}
                <strong
                  className={css`
                    font-weight: 600;
                  `}
                >
                  {destFileName}
                </strong>
              </>
            ) : null}
          </ModalTitle>
        </ModalHeader>
        <ModalBody
          className={css`
            padding-bottom: 0;
            overflow: hidden;
          `}
        >
          <div
            className={cx(
              css`
                border: 1px solid ${theme.colors.borderDarker};
                border-radius: 3px;
                overflow-y: scroll;
                overflow-x: hidden;
              `,
              isMobile
                ? css`
                    flex-grow: 1;
                  `
                : css`
                    height: 300px;
                  `
            )}
          >
            <RepoFilesDirPicker dirPickerId={dirPickerId} />
          </div>
        </ModalBody>
        <ModalFooter>
          <ModalFooterExtra>
            <Button
              type="button"
              disabled={!canShowCreateDir}
              onClick={() => createDirModal.show()}
            >
              Create folder
            </Button>
          </ModalFooterExtra>
          <ModalFooterButtons>
            <ModalFooterButton type="button" onClick={cancel}>
              Cancel
            </ModalFooterButton>
            <ModalFooterButton
              type="button"
              variant={canMove ? 'primary' : 'disabled'}
              disabled={!canMove}
              onClick={move}
            >
              {mode === 'Copy' ? 'Copy' : 'Move'}
            </ModalFooterButton>
          </ModalFooterButtons>
        </ModalFooter>

        <CreateDirModal
          isVisible={createDirModal.isVisible}
          canCreateDir={canCreateDir}
          createDir={createDir}
          hide={createDirModal.hide}
        />
      </>
    );
  }
);

export const RepoFilesMoveModal = memo(() => {
  const webVault = useWebVault();
  const info = useSubscribe(
    (v, cb) => v.repoFilesMoveInfoSubscribe(cb),
    (v) => v.repoFilesMoveInfoData,
    []
  );
  const cancel = useCallback(() => webVault.repoFilesMoveCancel(), [webVault]);

  return (
    <Modal show={info !== undefined} onHide={cancel}>
      {info !== undefined ? (
        <RepoFilesMoveModalContent info={info} cancel={cancel} />
      ) : (
        <></>
      )}
    </Modal>
  );
});
