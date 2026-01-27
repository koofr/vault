import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

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
  const intl = useIntl();
  const webVault = useWebVault();

  const autoLock = repo.autoLock;
  const options: {
    value: RepoAutoLockAfter['type'];
    label: string;
  }[] = [
    {
      value: 'NoLimit',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.no_limit',
        description:
          'Auto-lock dropdown option meaning no time limit. Full text: Lock after<new line>No time limit',
        defaultMessage: 'No time limit',
      }),
    },
    {
      value: 'Inactive1Minute',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.inactive_1_minute',
        description:
          'Auto-lock dropdown option for 1 minute of inactivity. Full text: Lock after<new line>1 minute of inactivity',
        defaultMessage: '1 minute of inactivity',
      }),
    },
    {
      value: 'Inactive5Mininutes',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.inactive_5_minutes',
        description:
          'Auto-lock dropdown option for 5 minutes of inactivity. Full text: Lock after<new line>5 minutes of inactivity',
        defaultMessage: '5 minutes of inactivity',
      }),
    },
    {
      value: 'Inactive10Minutes',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.inactive_10_minutes',
        description:
          'Auto-lock dropdown option for 10 minutes of inactivity. Full text: Lock after<new line>10 minutes of inactivity',
        defaultMessage: '10 minutes of inactivity',
      }),
    },
    {
      value: 'Inactive30Minutes',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.inactive_30_minutes',
        description:
          'Auto-lock dropdown option for 30 minutes of inactivity. Full text: Lock after<new line>30 minutes of inactivity',
        defaultMessage: '30 minutes of inactivity',
      }),
    },
    {
      value: 'Inactive1Hour',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.inactive_1_hour',
        description:
          'Auto-lock dropdown option for 1 hour of inactivity. Full text: Lock after<new line>1 hour of inactivity',
        defaultMessage: '1 hour of inactivity',
      }),
    },
    {
      value: 'Inactive2Hours',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.inactive_2_hours',
        description:
          'Auto-lock dropdown option for 2 hours of inactivity. Full text: Lock after<new line>2 hours of inactivity',
        defaultMessage: '2 hours of inactivity',
      }),
    },
    {
      value: 'Inactive4Hours',
      label: intl.formatMessage({
        id: 'web.repo_lock.lock_after_options.inactive_4_hours',
        description:
          'Auto-lock dropdown option for 4 hours of inactivity. Full text: Lock after<new line>4 hours of inactivity',
        defaultMessage: '4 hours of inactivity',
      }),
    },
  ];
  let customSeconds: number | undefined = undefined;
  if (autoLock.after?.type === 'Custom') {
    options.push({
      value: 'Custom',
      label: intl.formatMessage(
        {
          id: 'web.repo_lock.lock_after_options.custom',
          description:
            'Auto-lock dropdown option showing a custom number of inactivity seconds. Full text: Lock after<new line>Custom (N seconds)',
          defaultMessage:
            'Custom ({seconds, plural, one {# second} other {# seconds}})',
        },
        {
          seconds: autoLock.after.seconds,
        },
      ),
    });
    customSeconds = autoLock.after.seconds;
  }

  const updateAutoLock = (autoLock: RepoAutoLock) => {
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
        <FormattedMessage
          id="web.repo_lock.title"
          description="Section header for Safe Box auto-lock settings."
          defaultMessage="Lock Safe Box"
        />
      </h2>

      <p
        className={css`
          font-size: 13px;
          font-weight: normal;
          margin: 0 0 10px;
        `}
      >
        <FormattedMessage
          id="web.repo_lock.lock_after.heading"
          description="Label above the auto-lock timeout dropdown."
          defaultMessage="Lock after"
        />
        :
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
          aria-label={intl.formatMessage({
            id: 'web.repo_lock.lock_after_select.aria_label',
            description:
              'Accessibility label for the auto-lock timeout dropdown control.',
            defaultMessage: 'Lock Safe Box after',
          })}
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
        <FormattedMessage
          id="web.repo_lock.lock_on.button"
          description="Label above the lock condition checkboxes."
          defaultMessage="Lock on"
        />
        :
      </p>

      <div>
        <label
          className={css`
            display: flex;
            flex-direction: row;
            align-items: center;
          `}
          aria-label="Lock Safe Box on app hidden"
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
            <FormattedMessage
              id="web.repo_lock.lock_on_app_hidden.text"
              description="Checkbox label to lock the Safe Box when the app is hidden/backgrounded."
              defaultMessage="App hidden"
            />
          </span>
        </label>
      </div>

      {repo.state === 'Unlocked' ? (
        <div>
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
            <FormattedMessage
              id="web.repo_lock.lock_now.button"
              description="Button label to immediately lock the Safe Box."
              defaultMessage="Lock now"
            />
          </Button>
        </div>
      ) : null}
    </div>
  );
});
RepoLock.displayName = 'RepoLock';
