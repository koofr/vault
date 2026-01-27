import { css, cx } from '@emotion/css';
import { memo, useCallback, useState } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

import { Button } from '../../components/Button';
import { AutoFocusPasswordInput } from '../../components/PasswordInput';
import { useIsMobile } from '../../components/useIsMobile';
import { RepoUnlockInfo } from '../../vault-wasm/vault-wasm';

export const RepoUnlockForm = memo<{
  info: RepoUnlockInfo;
  onUnlock: (password: string) => void;
}>(({ info, onUnlock }) => {
  const intl = useIntl();
  const isMobile = useIsMobile();
  const [password, setPassword] = useState('');
  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();

      onUnlock(password);
    },
    [onUnlock, password],
  );

  return (
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
            `,
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
        <FormattedMessage
          id="web.repo_unlock_form.message"
          description="Instruction text above the Safe Key field on the unlock screen."
          defaultMessage="Enter your Safe Key to continue"
        />
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
            inputAriaLabel={intl.formatMessage({
              id: 'web.repo_unlock_form.password.aria_label',
              description:
                'Aria label for the password input field on the unlock screen.',
              defaultMessage: 'Safe Key',
            })}
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
          <FormattedMessage
            id="web.repo_unlock_form.submit.button"
            description="Primary button on the Safe Box unlock form."
            defaultMessage="Continue"
          />
        </Button>
      </form>
    </div>
  );
});
RepoUnlockForm.displayName = 'RepoUnlockForm';
