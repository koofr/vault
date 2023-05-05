import { css } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo, useState } from 'react';

import { Button } from '../../components/Button';
import { PasswordInput } from '../../components/PasswordInput';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoConfigInfo } from './RepoConfigInfo';

export const RepoConfigBackup = memo<{ repoId: string }>(({ repoId }) => {
  const webVault = useWebVault();
  useMemo(() => webVault.repoConfigBackupInit(repoId), [webVault, repoId]);
  useEffect(() => {
    return () => {
      webVault.repoConfigBackupDestroy(repoId);
    };
  }, [webVault, repoId]);
  const [info] = useSubscribe(
    (v, cb) => v.repoConfigBackupInfoSubscribe(cb),
    (v) => v.repoConfigBackupInfoData,
    []
  );
  const [password, setPassword] = useState('');
  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();

      webVault.repoConfigBackupGenerate(password);
    },
    [webVault, password]
  );

  if (info === undefined) {
    return null;
  }

  return (
    <div className={css``}>
      <h2
        className={css`
          font-size: 28px;
          font-weight: normal;
          margin: 0 0 20px;
        `}
      >
        Backup config
      </h2>
      {info.config === undefined ? (
        <form
          onSubmit={onSubmit}
          className={css`
            margin-bottom: 15px;
            max-width: 400px;
          `}
        >
          <div
            className={css`
              margin-bottom: 15px;
            `}
          >
            Enter your Safe Key:
          </div>
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
              margin-bottom: 15px;
            `}
          >
            <PasswordInput value={password} onChange={setPassword} />
          </div>
          <Button
            type="submit"
            variant={info.status.type === 'Loading' ? 'disabled' : 'primary'}
            disabled={info.status.type === 'Loading'}
          >
            Show
          </Button>
        </form>
      ) : (
        <RepoConfigInfo config={info.config} />
      )}
    </div>
  );
});
