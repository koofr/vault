import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import errorIconImage from '../assets/images/error-icon@2x.png';
import { Button } from './Button';

export const ErrorComponent = memo<{ error: string; onRetry?: () => void }>(
  ({ error, onRetry }) => {
    const theme = useTheme();

    return (
      <div
        className={css`
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 80px 0 0;
        `}
      >
        <img
          src={errorIconImage}
          alt=""
          className={css`
            display: block;
            width: 290px;
            height: 194px;
          `}
        />
        <h3
          className={css`
            font-size: 14px;
            color: ${theme.colors.text};
            font-weight: 600;
            margin: 0 0 20px;
          `}
        >
          {error}
        </h3>
        {onRetry !== undefined ? (
          <Button type="button" variant="primary" onClick={onRetry}>
            Try again
          </Button>
        ) : null}
      </div>
    );
  },
);
