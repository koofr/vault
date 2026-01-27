import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';
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

export const RemoteFilesDirPickerModalContent = memo<{
  dirPickerId: number;
  onClick: (pickerId: number, itemId: string, isArrow: boolean) => void;
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
          <ModalTitle>
            <FormattedMessage
              id="web.remote_files_dir_picker.title"
              description="Modal title for choosing a destination folder in remote files."
              defaultMessage="Select a folder"
            />
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
              <FormattedMessage
                id="web.remote_files_dir_picker.create_folder.button"
                description="Button label to create a new folder in the remote folder picker."
                defaultMessage="Create folder"
              />
            </Button>
          </ModalFooterExtra>
          <ModalFooterButtons>
            <ModalFooterButton type="button" onClick={cancel}>
              <FormattedMessage
                id="web.remote_files_dir_picker.cancel.button"
                description="Cancel button in the remote folder picker modal."
                defaultMessage="Cancel"
              />
            </ModalFooterButton>
            <ModalFooterButton
              type="button"
              variant={canSelect ? 'primary' : 'disabled'}
              disabled={!canSelect}
              onClick={select}
            >
              <FormattedMessage
                id="web.remote_files_dir_picker.select.button"
                description="Confirm button in the remote folder picker modal."
                defaultMessage="Select"
              />
            </ModalFooterButton>
          </ModalFooterButtons>
        </ModalFooter>
      </>
    );
  },
);
RemoteFilesDirPickerModalContent.displayName =
  'RemoteFilesDirPickerModalContent';

export const RemoteFilesDirPickerModal = memo<{
  dirPickerId?: number;
  onClick: (pickerId: number, itemId: string, isArrow: boolean) => void;
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
RemoteFilesDirPickerModal.displayName = 'RemoteFilesDirPickerModal';
