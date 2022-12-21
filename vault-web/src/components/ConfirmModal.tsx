import { memo } from 'react';

import {
  Modal,
  ModalBody,
  ModalFooter,
  ModalFooterButton,
  ModalFooterButtons,
  ModalHeader,
  ModalTitle,
} from './modal/Modal';

export interface ConfirmPayload {
  title: React.ReactNode;
  message: React.ReactNode;
  confirmText: string;
  cancelText?: string;
  onConfirm: () => void;
}

export const ConfirmModalContent = memo<
  ConfirmPayload & {
    hide: () => void;
  }
>(({ title, message, confirmText, cancelText, onConfirm, hide }) => {
  return (
    <>
      <ModalHeader>
        <ModalTitle>{title}</ModalTitle>
      </ModalHeader>
      <ModalBody>{message}</ModalBody>
      <ModalFooter>
        <ModalFooterButtons>
          <ModalFooterButton type="button" onClick={hide}>
            {cancelText ?? 'Cancel'}
          </ModalFooterButton>
          <ModalFooterButton
            type="button"
            variant="primary"
            onClick={() => {
              hide();

              onConfirm();
            }}
          >
            {confirmText}
          </ModalFooterButton>
        </ModalFooterButtons>
      </ModalFooter>
    </>
  );
});

export const ConfirmModal = memo<{
  isVisible: boolean;
  payload: ConfirmPayload | undefined;
  hide: () => void;
}>(({ isVisible, payload, hide }) => {
  return (
    <Modal show={isVisible} onHide={hide}>
      {isVisible && payload !== undefined ? (
        <ConfirmModalContent {...payload} hide={hide} />
      ) : (
        <></>
      )}
    </Modal>
  );
});
