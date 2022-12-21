import { memo, useCallback, useMemo, useState } from 'react';

import { selectFilenameRange } from '../utils/selectRange';

import { TextInput } from './TextInput';
import {
  Modal,
  ModalBody,
  ModalFooter,
  ModalFooterButton,
  ModalFooterButtons,
  ModalHeader,
  ModalTitle,
} from './modal/Modal';

export interface RenameFilePayload {
  originalName: string;
  isDir: boolean;
  canRenameFile: (name: string) => boolean;
  renameFile: (name: string) => void;
}

export const RenameFileModalContent = memo<
  RenameFilePayload & {
    hide: () => void;
  }
>(({ originalName, isDir, canRenameFile, renameFile, hide }) => {
  const [name, setName] = useState(originalName);
  const canSubmit = useMemo(() => canRenameFile(name), [canRenameFile, name]);
  const nameRef = useCallback(
    (el: HTMLInputElement | null) => {
      if (el !== null) {
        selectFilenameRange(el, isDir);
      }
    },
    [isDir]
  );

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();

        hide();

        renameFile(name);
      }}
    >
      <ModalHeader>
        <ModalTitle>Enter new name for '{originalName}'</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <TextInput
          type="text"
          placeholder="New name"
          value={name}
          onChange={(event) => setName(event.currentTarget.value)}
          ref={nameRef}
        />
      </ModalBody>
      <ModalFooter>
        <ModalFooterButtons>
          <ModalFooterButton type="button" onClick={hide}>
            Cancel
          </ModalFooterButton>
          <ModalFooterButton
            type="submit"
            variant={canSubmit ? 'primary' : 'disabled'}
            disabled={!canSubmit}
          >
            Rename
          </ModalFooterButton>
        </ModalFooterButtons>
      </ModalFooter>
    </form>
  );
});

export const RenameFileModal = memo<{
  isVisible: boolean;
  payload: RenameFilePayload | undefined;
  hide: () => void;
}>(({ isVisible, payload, hide }) => {
  return (
    <Modal show={isVisible} onHide={hide}>
      {isVisible && payload !== undefined ? (
        <RenameFileModalContent {...payload} hide={hide} />
      ) : (
        <></>
      )}
    </Modal>
  );
});
