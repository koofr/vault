import { css } from '@emotion/css';
import { memo, useCallback, useState } from 'react';

import { TextInput } from '../../components/TextInput';
import { Modal, ModalHeader } from '../../components/modal/Modal';
import { ModalTitle } from '../../components/modal/Modal';
import { ModalBody } from '../../components/modal/Modal';
import { ModalFooter } from '../../components/modal/Modal';
import { ModalFooterButtons } from '../../components/modal/Modal';
import { ModalFooterButton } from '../../components/modal/Modal';
import { selectRange } from '../../utils/selectRange';
import { Dialog as VaultWasmDialog } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

const DialogInner = memo<{
  dialogId: number;
  dialog: VaultWasmDialog;
  dialogRef: { current: VaultWasmDialog };
}>(({ dialogId, dialog, dialogRef }) => {
  const webVault = useWebVault();
  const confirm = useCallback(() => {
    webVault.dialogsConfirm(dialogId);
  }, [webVault, dialogId]);
  const cancel = useCallback(() => {
    webVault.dialogsCancel(dialogId);
  }, [webVault, dialogId]);
  const [localInputValue, setLocalInputValue] = useState(dialog.inputValue);
  const setInputValue = useCallback(
    (value: string) => {
      setLocalInputValue(value);

      webVault.dialogsSetInputValue(dialogId, value);
    },
    [webVault, dialogId],
  );
  const inputRef = useCallback(
    (el: HTMLInputElement | null) => {
      if (el !== null) {
        if (
          dialogRef.current !== undefined &&
          dialogRef.current.inputValueSelected !== undefined
        ) {
          selectRange(el, 0, dialogRef.current.inputValueSelected.length);
        } else {
          el.focus();
        }
      }
    },
    [dialogRef],
  );
  const {
    typ,
    title,
    message,
    inputPlaceholder,
    confirmButtonText,
    confirmButtonEnabled,
    confirmButtonStyle,
    cancelButtonText,
  } = dialog;

  return (
    <Modal show onHide={cancel}>
      <form
        className={css`
          display: flex;
          flex-direction: column;
        `}
        onSubmit={(e) => {
          e.preventDefault();
          confirm();
        }}
      >
        <ModalHeader>
          <ModalTitle>{title}</ModalTitle>
        </ModalHeader>
        <ModalBody>
          {message !== undefined ? (
            <p
              className={css`
                margin: 0 0 ${typ === 'Prompt' ? '15px' : '0'};
              `}
            >
              {message}
            </p>
          ) : null}
          {typ === 'Prompt' ? (
            <TextInput
              type="text"
              placeholder={inputPlaceholder}
              value={localInputValue}
              onChange={(e) => setInputValue(e.currentTarget.value)}
              ref={inputRef}
            />
          ) : null}
        </ModalBody>
        <ModalFooter>
          <ModalFooterButtons>
            {cancelButtonText !== undefined ? (
              <ModalFooterButton type="button" onClick={cancel}>
                {cancelButtonText}
              </ModalFooterButton>
            ) : null}
            <ModalFooterButton
              type="submit"
              variant={
                confirmButtonEnabled
                  ? confirmButtonStyle === 'Destructive'
                    ? 'destructive'
                    : 'primary'
                  : 'disabled'
              }
              onClick={confirm}
              disabled={!confirmButtonEnabled}
            >
              {confirmButtonText}
            </ModalFooterButton>
          </ModalFooterButtons>
        </ModalFooter>
      </form>
    </Modal>
  );
});

export const Dialog = memo<{ dialogId: number }>(({ dialogId }) => {
  const [dialog, dialogRef] = useSubscribe(
    (v, cb) => v.dialogsDialogSubscribe(dialogId, cb),
    (v) => v.dialogsDialogData,
    [dialogId],
  );

  if (dialog === undefined || dialogRef.current === undefined) {
    return null;
  }

  return (
    <DialogInner
      dialogId={dialogId}
      dialog={dialog}
      dialogRef={dialogRef as { current: VaultWasmDialog }}
    />
  );
});
