import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback, useState } from 'react';
import { FormattedMessage } from 'react-intl';
import { useNavigate } from 'react-router-dom';

import { Button } from '../../components/Button';
import { RepoCreated } from '../../vault-wasm/vault-wasm';

import { RepoConfigInfo } from '../repo/RepoConfigInfo';

export const RepoCreateCreatedComponent = memo<{ created: RepoCreated }>(
  ({ created }) => {
    const theme = useTheme();
    const navigate = useNavigate();
    const [configSaved, setConfigSaved] = useState(false);
    const openRepo = useCallback(() => {
      if (created !== undefined) {
        // eslint-disable-next-line @typescript-eslint/no-floating-promises
        navigate(`/repos/${created.repoId}`);
      }
    }, [navigate, created]);

    return (
      <div>
        <h1
          className={css`
            font-size: 28px;
            font-weight: normal;
            margin: 0 0 20px;
          `}
        >
          <FormattedMessage
            id="web.repo_create.created.title"
            description="Success headline shown after a Safe Box is created."
            defaultMessage="Your Safe Box has been created"
          />
        </h1>
        <p
          className={css`
            margin: 0 0 20px;
          `}
        >
          <FormattedMessage
            id="web.repo_create.created.description"
            description="Instruction text prompting users to save the Safe Box configuration before continuing."
            defaultMessage="Before you start using your Safe Box please safely store the configuration."
          />
        </p>
        <div
          className={css`
            border-bottom: 1px solid ${theme.colors.border};
            margin-bottom: 25px;
          `}
        />
        <div
          className={css`
            margin-bottom: 25px;
          `}
          onMouseDown={() => setConfigSaved(true)}
        >
          <RepoConfigInfo config={created.config} />
        </div>
        <div
          className={css`
            border-bottom: 1px solid ${theme.colors.border};
            margin-bottom: 25px;
          `}
        />
        <Button
          type="button"
          variant={configSaved ? 'primary' : 'disabled'}
          disabled={!configSaved}
          onClick={openRepo}
        >
          <FormattedMessage
            id="web.repo_create.created.continue.button"
            description="Button label on the create-success screen to continue to the Safe Box."
            defaultMessage="Continue"
          />
        </Button>
      </div>
    );
  },
);
RepoCreateCreatedComponent.displayName = 'RepoCreateCreatedComponent';
