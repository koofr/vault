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

export const CreateDirModalContent = memo<{
  canCreateDir: (name: string) => boolean;
  createDir: (name: string) => void;
  hide: () => void;
}>(({ canCreateDir, createDir, hide }) => {
  const [name, setName] = useState('New folder');
  const canSubmit = useMemo(() => canCreateDir(name), [canCreateDir, name]);
  const nameRef = useCallback((el: HTMLInputElement | null) => {
    if (el !== null) {
      selectFilenameRange(el, true);
    }
  }, []);

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();

        hide();

        createDir(name);
      }}
    >
      <ModalHeader>
        <ModalTitle>Enter new folder name</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <TextInput
          type="text"
          placeholder="Folder name"
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
            Create folder
          </ModalFooterButton>
        </ModalFooterButtons>
      </ModalFooter>
    </form>
  );
});

export const CreateDirModal = memo<{
  isVisible: boolean;
  canCreateDir: (name: string) => boolean;
  createDir: (name: string) => void;
  hide: () => void;
}>(({ isVisible, canCreateDir, createDir, hide }) => {
  return (
    <Modal show={isVisible} onHide={hide}>
      {isVisible ? (
        <CreateDirModalContent
          canCreateDir={canCreateDir}
          createDir={createDir}
          hide={hide}
        />
      ) : (
        <></>
      )}
    </Modal>
  );
});
