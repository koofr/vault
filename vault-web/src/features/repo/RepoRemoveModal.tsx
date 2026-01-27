import { css } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo, useState } from 'react';
import { FormattedMessage } from 'react-intl';
import { useNavigate } from 'react-router-dom';

import { AutoFocusPasswordInput } from '../../components/PasswordInput';
import {
  Modal,
  ModalBody,
  ModalFooter,
  ModalFooterButton,
  ModalFooterButtons,
  ModalHeader,
  ModalTitle,
} from '../../components/modal/Modal';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

export const RepoRemoveModalContent = memo<{
  repoId: string;
  hide: () => void;
}>(({ repoId, hide }) => {
  const webVault = useWebVault();
  const navigate = useNavigate();
  const removeId = useMemo(
    () => webVault.repoRemoveCreate(repoId),
    [webVault, repoId],
  );
  useEffect(() => {
    return () => {
      webVault.repoRemoveDestroy(removeId);
    };
  }, [webVault, removeId]);
  const [info] = useSubscribe(
    (v, cb) => v.repoRemoveInfoSubscribe(removeId, cb),
    (v) => v.repoRemoveInfoData,
    [removeId],
  );
  const [password, setPassword] = useState('');
  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();

      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      (async () => {
        const success = await webVault.repoRemoveRemove(removeId, password);

        if (success) {
          await navigate('/');
        }
      })();
    },
    [webVault, removeId, password, navigate],
  );

  if (info === undefined) {
    return null;
  }

  return (
    <form onSubmit={onSubmit}>
      <ModalHeader>
        <ModalTitle>
          <FormattedMessage
            id="web.repo_remove.title"
            description="Modal title for the destructive action that removes a Safe Box configuration."
            defaultMessage="Destroy Safe Box"
          />
        </ModalTitle>
      </ModalHeader>
      <ModalBody
        className={css`
          padding-bottom: 0;
        `}
      >
        <FormattedMessage
          id="web.repo_remove.message"
          description="Body text in the destroy Safe Box modal explaining consequences and requesting the Safe Key."
          defaultMessage="<p>Do you really want to destroy Safe Box <b>{name}</b>?</p><p>Destroying the Safe Box will keep all the files on Koofr but remove the configuration so you won't be able to decrypt the files if you didn't save the configuration.</p><p><b>This action cannot be undone.</b></p><p>Enter your Safe Key to confirm the removal:</p>"
          values={{
            name: info.repoName,
            b: (chunks) => (
              <strong
                className={css`
                  font-weight: 600;
                `}
              >
                {chunks}
              </strong>
            ),
            p: (chunks) => (
              <p
                className={css`
                  margin: 0 0 20px;
                `}
              >
                {chunks}
              </p>
            ),
          }}
        />

        {info.status.type === 'Error' ? (
          <div
            className={css`
              background-color: #fbedeb;
              padding: 6px 15px;
              border-radius: 3px;
              margin: 0 0 15px;
            `}
          >
            {info.status.error}
          </div>
        ) : null}

        <div
          className={css`
            display: flex;
            flex-direction: row;
          `}
        >
          <AutoFocusPasswordInput value={password} onChange={setPassword} />
        </div>
      </ModalBody>
      <ModalFooter>
        <ModalFooterButtons>
          <ModalFooterButton type="button" onClick={hide}>
            <FormattedMessage
              id="web.repo_remove.cancel.button"
              description="Cancel button in the destroy Safe Box modal."
              defaultMessage="Cancel"
            />
          </ModalFooterButton>
          <ModalFooterButton
            type="submit"
            variant={
              info.status.type === 'Loading' ? 'disabled' : 'destructive'
            }
            disabled={info.status.type === 'Loading'}
          >
            <FormattedMessage
              id="web.repo_remove.confirm.button"
              description="Destructive confirmation button in the destroy Safe Box modal."
              defaultMessage="Destroy"
            />
          </ModalFooterButton>
        </ModalFooterButtons>
      </ModalFooter>
    </form>
  );
});
RepoRemoveModalContent.displayName = 'RepoRemoveModalContent';

export const RepoRemoveModal = memo<{
  repoId?: string;
  hide: () => void;
}>(({ repoId, hide }) => {
  return (
    <Modal show={repoId !== undefined} onHide={hide}>
      {repoId !== undefined ? (
        <RepoRemoveModalContent repoId={repoId} hide={hide} />
      ) : (
        <></>
      )}
    </Modal>
  );
});
RepoRemoveModal.displayName = 'RepoRemoveModal';
