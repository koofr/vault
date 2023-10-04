import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback, useState } from 'react';
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
          Your Safe Box has been created.
        </h1>
        <p
          className={css`
            margin: 0 0 20px;
          `}
        >
          Before you start using your Safe Box please safely store the
          configuration.
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
          Continue
        </Button>
      </div>
    );
  },
);
