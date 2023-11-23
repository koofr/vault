import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback } from 'react';

import { Button } from '../../components/Button';
import { DirPicker } from '../../components/dirpicker/DirPicker';
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
import { RepoFilesMoveInfo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

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
      createDirEnabled,
      canMove,
    },
    cancel,
  }) => {
    const isMobile = useIsMobile();
    const theme = useTheme();
    const webVault = useWebVault();
    const dirPickerOnClick = useCallback(
      (_: number, itemId: string, isArrow: boolean) =>
        webVault.repoFilesMoveDirPickerClick(itemId, isArrow),
      [webVault],
    );
    const createDir = useCallback(() => {
      webVault.repoFilesMoveCreateDir();
    }, [webVault]);
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
                  `,
            )}
          >
            <DirPicker pickerId={dirPickerId} onClick={dirPickerOnClick} />
          </div>
        </ModalBody>
        <ModalFooter>
          <ModalFooterExtra>
            <Button
              type="button"
              disabled={!createDirEnabled}
              onClick={createDir}
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
      </>
    );
  },
);

export const RepoFilesMoveModal = memo(() => {
  const webVault = useWebVault();
  const [info] = useSubscribe(
    (v, cb) => v.repoFilesMoveInfoSubscribe(cb),
    (v) => v.repoFilesMoveInfoData,
    [],
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
