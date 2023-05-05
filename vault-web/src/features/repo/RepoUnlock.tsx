import { css, cx } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo, useState } from 'react';

import { Button } from '../../components/Button';
import { AutoFocusPasswordInput } from '../../components/PasswordInput';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useIsMobile } from '../../components/useIsMobile';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

export const RepoUnlock = memo<{ repoId: string }>(({ repoId }) => {
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  useMemo(() => webVault.repoUnlockInit(repoId), [webVault, repoId]);
  useEffect(() => {
    return () => {
      webVault.repoUnlockDestroy(repoId);
    };
  }, [webVault, repoId]);
  const [info] = useSubscribe(
    (v, cb) => v.repoUnlockInfoSubscribe(cb),
    (v) => v.repoUnlockInfoData,
    []
  );
  const [password, setPassword] = useState('');
  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();

      webVault.repoUnlockUnlock(password);
    },
    [webVault, password]
  );
  const navbarHeader = useSingleNavbarBreadcrumb(info?.repoName ?? '');

  if (info === undefined) {
    return null;
  }

  return (
    <DashboardLayout navbarHeader={navbarHeader}>
      <div
        className={cx(
          css`
            display: flex;
            flex-direction: column;
            align-items: center;
            text-align: center;
          `,
          isMobile
            ? css`
                padding: 0;
              `
            : css`
                padding: 50px 218px 0 0;

                @media (max-width: 1024px) {
                  padding: 50px 0 0;
                }
              `
        )}
      >
        <h1
          className={css`
            font-size: 32px;
            line-height: 42px;
            font-weight: normal;
            margin: 0 0 25px;
          `}
        >
          {info?.repoName}
        </h1>

        <h3
          className={css`
            font-size: 20px;
            line-height: 26px;
            font-weight: 300;
            margin: 0 0 30px;
          `}
        >
          Enter your Safe Key to unlock
        </h3>

        <form onSubmit={onSubmit}>
          {info.status.type === 'Error' ? (
            <div
              className={css`
                background-color: #fbedeb;
                padding: 6px 15px;
                border-radius: 3px;
                margin-bottom: 15px;
              `}
            >
              {info.status.error}
            </div>
          ) : null}

          <div
            className={css`
              margin-bottom: 20px;
            `}
          >
            <AutoFocusPasswordInput
              value={password}
              onChange={setPassword}
              inputAriaLabel="Safe Key"
            />
          </div>
          <Button
            type="submit"
            variant={info.status.type === 'Loading' ? 'disabled' : 'primary'}
            disabled={info.status.type === 'Loading'}
            className={css`
              font-size: 16px;
              width: 250px;
            `}
          >
            Unlock
          </Button>
        </form>
      </div>
    </DashboardLayout>
  );
});
