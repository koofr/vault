import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback } from 'react';
import { FormattedMessage } from 'react-intl';

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
    const move = useCallback(() => {
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
            {destFileName !== undefined ? (
              mode === 'Copy' ? (
                <FormattedMessage
                  id="web.repo_files_move.copy.title"
                  description="Modal title when copying selected items to a destination folder."
                  defaultMessage="Copy <b>{count, plural, one {# item} other {# items}}</b> to <b>{dest}</b>"
                  values={{
                    count: srcFilesCount,
                    dest: destFileName,
                    b: (chunks) => (
                      <strong
                        className={css`
                          font-weight: 600;
                        `}
                      >
                        {chunks}
                      </strong>
                    ),
                  }}
                />
              ) : (
                <FormattedMessage
                  id="web.repo_files_move.move.title"
                  description="Modal title when moving selected items to a destination folder."
                  defaultMessage="Move <b>{count, plural, one {# item} other {# items}}</b> to <b>{dest}</b>"
                  values={{
                    count: srcFilesCount,
                    dest: destFileName,
                    b: (chunks) => (
                      <strong
                        className={css`
                          font-weight: 600;
                        `}
                      >
                        {chunks}
                      </strong>
                    ),
                  }}
                />
              )
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
              <FormattedMessage
                id="web.repo_files_move.create_dir.button"
                description="Button label in the move/copy modal to create a new destination folder."
                defaultMessage="Create folder"
              />
            </Button>
          </ModalFooterExtra>
          <ModalFooterButtons>
            <ModalFooterButton type="button" onClick={cancel}>
              <FormattedMessage
                id="web.repo_files_move.cancel.button"
                description="Cancel button in the move/copy modal."
                defaultMessage="Cancel"
              />
            </ModalFooterButton>
            <ModalFooterButton
              type="button"
              variant={canMove ? 'primary' : 'disabled'}
              disabled={!canMove}
              onClick={move}
            >
              {mode === 'Copy' ? (
                <FormattedMessage
                  id="web.repo_files_move.copy.button"
                  description="Confirm button label in the move/copy modal when in copy mode."
                  defaultMessage="Copy"
                />
              ) : (
                <FormattedMessage
                  id="web.repo_files_move.move.button"
                  description="Confirm button label in the move/copy modal when in move mode."
                  defaultMessage="Move"
                />
              )}
            </ModalFooterButton>
          </ModalFooterButtons>
        </ModalFooter>
      </>
    );
  },
);
RepoFilesMoveModalContent.displayName = 'RepoFilesMoveModalContent';

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
RepoFilesMoveModal.displayName = 'RepoFilesMoveModal';
