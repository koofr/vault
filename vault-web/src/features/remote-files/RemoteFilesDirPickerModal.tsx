import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import { Button } from '../../components/Button';
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

import { DirPicker } from '../../components/dirpicker/DirPicker';

export const RemoteFilesDirPickerModalContent = memo<{
  dirPickerId: number;
  onClick: (
    pickerId: number,
    itemId: string,
    isArrow: boolean,
  ) => Promise<void>;
  canSelect: boolean;
  select: () => void;
  cancel: () => void;
  createDirEnabled: boolean;
  createDir: () => void;
}>(
  ({
    dirPickerId,
    onClick,
    canSelect,
    select,
    cancel,
    createDirEnabled,
    createDir,
  }) => {
    const isMobile = useIsMobile();
    const theme = useTheme();

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
                  `,
            )}
          >
            <DirPicker pickerId={dirPickerId} onClick={onClick} />
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
              variant={canSelect ? 'primary' : 'disabled'}
              disabled={!canSelect}
              onClick={select}
            >
              Select
            </ModalFooterButton>
          </ModalFooterButtons>
        </ModalFooter>
      </>
    );
  },
);

export const RemoteFilesDirPickerModal = memo<{
  dirPickerId?: number;
  onClick: (
    pickerId: number,
    itemId: string,
    isArrow: boolean,
  ) => Promise<void>;
  canSelect: boolean;
  select: () => void;
  cancel: () => void;
  createDirEnabled: boolean;
  createDir: () => void;
}>(
  ({
    dirPickerId,
    onClick,
    canSelect,
    select,
    cancel,
    createDirEnabled,
    createDir,
  }) => {
    return (
      <Modal show={dirPickerId !== undefined} onHide={cancel}>
        {dirPickerId !== undefined ? (
          <RemoteFilesDirPickerModalContent
            dirPickerId={dirPickerId}
            onClick={onClick}
            canSelect={canSelect}
            select={select}
            cancel={cancel}
            createDirEnabled={createDirEnabled}
            createDir={createDir}
          />
        ) : (
          <></>
        )}
      </Modal>
    );
  },
);
