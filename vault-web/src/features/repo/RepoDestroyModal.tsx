import { css } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo, useState } from 'react';
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

export const RepoDestroyModalContent = memo<{
  repoId: string;
  hide: () => void;
}>(({ repoId, hide }) => {
  const webVault = useWebVault();
  const navigate = useNavigate();
  useMemo(() => webVault.repoRemoveInit(repoId), [webVault, repoId]);
  useEffect(() => {
    return () => {
      webVault.repoRemoveDestroy(repoId);
    };
  }, [webVault, repoId]);
  const [info] = useSubscribe(
    (v, cb) => v.repoRemoveInfoSubscribe(cb),
    (v) => v.repoRemoveInfoData,
    []
  );
  const [password, setPassword] = useState('');
  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();

      (async () => {
        const success = await webVault.repoRemoveRemove(password);

        if (success) {
          navigate('/');
        }
      })();
    },
    [webVault, password, navigate]
  );

  if (info === undefined) {
    return null;
  }

  return (
    <form onSubmit={onSubmit}>
      <ModalHeader>
        <ModalTitle>Destroy Safe Box</ModalTitle>
      </ModalHeader>
      <ModalBody
        className={css`
          padding-bottom: 0;
        `}
      >
        <p
          className={css`
            margin: 0 0 20px;
          `}
        >
          Do you really want to destroy Safe Box{' '}
          <strong
            className={css`
              font-weight: 600;
            `}
          >
            {info.repoName}
          </strong>
          ?
        </p>
        <p
          className={css`
            margin: 0 0 20px;
          `}
        >
          Destroying the Safe Box will keep all the files on Koofr but remove
          the configuration so you won't be able to decrypt the files if you
          didn't save the configuration.
        </p>
        <p
          className={css`
            margin: 0 0 20px;
            font-weight: 600;
          `}
        >
          This action cannot be undone.
        </p>
        <p
          className={css`
            margin: 0 0 20px;
          `}
        >
          Enter your Safe Key to confirm the removal:
        </p>

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
            Cancel
          </ModalFooterButton>
          <ModalFooterButton
            type="submit"
            variant={
              info.status.type === 'Loading' ? 'disabled' : 'destructive'
            }
            disabled={info.status.type === 'Loading'}
          >
            Destroy
          </ModalFooterButton>
        </ModalFooterButtons>
      </ModalFooter>
    </form>
  );
});

export const RepoDestroyModal = memo<{
  repoId?: string;
  hide: () => void;
}>(({ repoId, hide }) => {
  return (
    <Modal show={repoId !== undefined} onHide={hide}>
      {repoId !== undefined ? (
        <RepoDestroyModalContent repoId={repoId} hide={hide} />
      ) : (
        <></>
      )}
    </Modal>
  );
});
