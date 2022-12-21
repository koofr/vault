import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

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

import { RemoteFilesDirPicker } from './RemoteFilesDirPicker';

export interface RemoteFilesDirPickerModalPayload {
  onSelect: (mountId: string, path: string) => void;
}

export const RemoteFilesDirPickerModalContent = memo<{
  dirPickerId: number;
  canSelect: boolean;
  select: () => void;
  cancel: () => void;
  canShowCreateDir: boolean;
  canCreateDir: (name: string) => boolean;
  createDir: (name: string) => void;
}>(
  ({
    dirPickerId,
    canSelect,
    select,
    cancel,
    canShowCreateDir,
    canCreateDir,
    createDir,
  }) => {
    const isMobile = useIsMobile();
    const theme = useTheme();
    const createDirModal = useModal();

    return (
      <>
        <ModalHeader>
          <ModalTitle>Select a folder</ModalTitle>
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
            <RemoteFilesDirPicker dirPickerId={dirPickerId} />
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
              variant={canSelect ? 'primary' : 'disabled'}
              disabled={!canSelect}
              onClick={select}
            >
              Select
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

export const RemoteFilesDirPickerModal = memo<{
  dirPickerId?: number;
  canSelect: boolean;
  select: () => void;
  cancel: () => void;
  canShowCreateDir: boolean;
  canCreateDir: (name: string) => boolean;
  createDir: (name: string) => void;
}>(
  ({
    dirPickerId,
    canSelect,
    select,
    cancel,
    canShowCreateDir,
    canCreateDir,
    createDir,
  }) => {
    return (
      <Modal show={dirPickerId !== undefined} onHide={cancel}>
        {dirPickerId !== undefined ? (
          <RemoteFilesDirPickerModalContent
            dirPickerId={dirPickerId}
            canSelect={canSelect}
            select={select}
            cancel={cancel}
            canShowCreateDir={canShowCreateDir}
            canCreateDir={canCreateDir}
            createDir={createDir}
          />
        ) : (
          <></>
        )}
      </Modal>
    );
  }
);
