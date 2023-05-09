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

import { RemoteFilesDirPicker } from './RemoteFilesDirPicker';

export interface RemoteFilesDirPickerModalPayload {
  onSelect: (mountId: string, path: string) => void;
}

export const RemoteFilesDirPickerModalContent = memo<{
  dirPickerId: number;
  canSelect: boolean;
  select: () => void;
  cancel: () => void;
  createDirEnabled: boolean;
  createDir: () => void;
}>(
  ({ dirPickerId, canSelect, select, cancel, createDirEnabled, createDir }) => {
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
  }
);

export const RemoteFilesDirPickerModal = memo<{
  dirPickerId?: number;
  canSelect: boolean;
  select: () => void;
  cancel: () => void;
  createDirEnabled: boolean;
  createDir: () => void;
}>(
  ({ dirPickerId, canSelect, select, cancel, createDirEnabled, createDir }) => {
    return (
      <Modal show={dirPickerId !== undefined} onHide={cancel}>
        {dirPickerId !== undefined ? (
          <RemoteFilesDirPickerModalContent
            dirPickerId={dirPickerId}
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
  }
);
