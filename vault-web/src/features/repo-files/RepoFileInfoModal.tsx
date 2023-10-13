import { memo } from 'react';

import {
  Modal,
  ModalBody,
  ModalFooter,
  ModalFooterButton,
  ModalFooterButtons,
  ModalHeader,
  ModalTitle,
} from '../../components/modal/Modal';
import { RepoFile } from '../../vault-wasm/vault-wasm';

import { RepoFileInfoGeneral } from './RepoFileInfoGeneral';
import { RepoFileInfoImage } from './RepoFileInfoImage';

export const RepoFileInfoModalContent = memo<{
  file: RepoFile;
  hide: () => void;
}>(({ file, hide }) => {
  return (
    <>
      <ModalHeader>
        <ModalTitle>Info</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <RepoFileInfoGeneral file={file} />
        <RepoFileInfoImage file={file} />
      </ModalBody>
      <ModalFooter>
        <ModalFooterButtons>
          <ModalFooterButton variant="primary" onClick={hide}>
            Close
          </ModalFooterButton>
        </ModalFooterButtons>
      </ModalFooter>
    </>
  );
});

export const RepoFileInfoModal = memo<{
  file?: RepoFile;
  hide: () => void;
}>(({ file, hide }) => {
  return (
    <Modal show={file !== undefined} onHide={hide}>
      {file !== undefined ? (
        <RepoFileInfoModalContent file={file} hide={hide} />
      ) : (
        <></>
      )}
    </Modal>
  );
});
