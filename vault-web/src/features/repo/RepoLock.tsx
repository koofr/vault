import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useState } from 'react';

import { Button } from '../../components/Button';
import { Checkbox } from '../../components/Checkbox';
import {
  Repo,
  RepoAutoLock,
  RepoAutoLockAfter,
} from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

export const RepoLock = memo<{ repo: Repo }>(({ repo }) => {
  const theme = useTheme();
  const webVault = useWebVault();

  const [autoLock, setAutoLock] = useState(repo.autoLock);
  let options: {
    value: RepoAutoLockAfter['type'];
    label: string;
  }[] = [
    { value: 'NoLimit', label: 'No time limit' },
    { value: 'Inactive1Minute', label: '1 minute of inactivity' },
    { value: 'Inactive5Mininutes', label: '5 minutes of inactivity' },
    { value: 'Inactive10Minutes', label: '10 minutes of inactivity' },
    { value: 'Inactive30Minutes', label: '30 minutes of inactivity' },
    { value: 'Inactive1Hour', label: '1 hour of inactivity' },
    { value: 'Inactive2Hours', label: '2 hours of inactivity' },
    { value: 'Inactive4Hours', label: '4 hours of inactivity' },
  ];
  let customSeconds: number | undefined = undefined;
  if (autoLock.after?.type === 'Custom') {
    options.push({
      value: 'Custom',
      label: `Custom (${autoLock.after.seconds} seconds)`,
    });
    customSeconds = autoLock.after.seconds;
  }

  const updateAutoLock = (autoLock: RepoAutoLock) => {
    setAutoLock(autoLock);

    webVault.reposSetAutoLock(repo.id, autoLock);
  };

  return (
    <div>
      <h2
        className={css`
          font-size: 28px;
          font-weight: normal;
          margin: 0 0 10px;
        `}
      >
        Lock Safe Box
      </h2>

      <p
        className={css`
          font-size: 13px;
          font-weight: normal;
          margin: 0 0 10px;
        `}
      >
        Lock after:
      </p>

      <div
        className={css`
          display: flex;
          flex-direction: row;
          align-items: center;
          margin: 0 0 20px;
        `}
      >
        <select
          value={autoLock.after.type}
          onChange={(e) => {
            const afterType = e.currentTarget
              .value as RepoAutoLockAfter['type'];

            let after: RepoAutoLockAfter;

            if (afterType === 'Custom') {
              after = { type: 'Custom', seconds: customSeconds! };
            } else {
              after = { type: afterType };
            }

            updateAutoLock({
              ...autoLock,
              after,
            });
          }}
          className={css`
            border: 1px solid ${theme.colors.borderDark};
            border-radius: 3px;
            height: 36px;
            width: 280px;
            padding: 0 7px;
          `}
        >
          {options.map(({ value, label }) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </div>

      <p
        className={css`
          font-size: 13px;
          font-weight: normal;
          margin: 0 0 10px;
        `}
      >
        Lock on:
      </p>

      <div
        className={css`
          //
        `}
      >
        <label
          className={css`
            display: flex;
            flex-direction: row;
            align-items: center;
          `}
        >
          <Checkbox
            value={autoLock.onAppHidden ? 'checked' : 'unchecked'}
            small
            onClick={() => {
              updateAutoLock({
                ...autoLock,
                onAppHidden: !autoLock.onAppHidden,
              });
            }}
          />

          <span
            className={css`
              margin-left: 8px;
            `}
          >
            App hidden
          </span>
        </label>
      </div>

      {repo.state === 'Unlocked' ? (
        <div className={css``}>
          <Button
            type="button"
            variant="primary"
            onClick={() => {
              webVault.reposLockRepo(repo.id);
            }}
            className={css`
              height: 36px;
              margin-top: 15px;
            `}
          >
            Lock now
          </Button>
        </div>
      ) : null}
    </div>
  );
});
