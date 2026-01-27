import { memo } from 'react';
import { FormattedMessage } from 'react-intl';

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
        <ModalTitle>
          <FormattedMessage
            id="web.repo_file_info.title"
            description="Modal title for the file info dialog/sheet."
            defaultMessage="Info"
          />
        </ModalTitle>
      </ModalHeader>
      <ModalBody>
        <RepoFileInfoGeneral file={file} />
        <RepoFileInfoImage file={file} />
      </ModalBody>
      <ModalFooter>
        <ModalFooterButtons>
          <ModalFooterButton variant="primary" onClick={hide}>
            <FormattedMessage
              id="web.repo_file_info.close.button"
              description="Close button label in the file info modal."
              defaultMessage="Close"
            />
          </ModalFooterButton>
        </ModalFooterButtons>
      </ModalFooter>
    </>
  );
});
RepoFileInfoModalContent.displayName = 'RepoFileInfoModalContent';

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
RepoFileInfoModal.displayName = 'RepoFileInfoModal';
