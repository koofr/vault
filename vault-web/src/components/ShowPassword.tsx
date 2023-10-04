import { css, cx } from '@emotion/css';
import { memo } from 'react';

import HidePasswordHoverIcon from '../assets/images/hide-password-hover.svg?react';
import HidePasswordIcon from '../assets/images/hide-password.svg?react';
import ShowPasswordHoverIcon from '../assets/images/show-password-hover.svg?react';
import ShowPasswordIcon from '../assets/images/show-password.svg?react';
import { buttonReset } from '../styles/mixins/buttons';

export const ShowPassword = memo<{
  value: boolean;
  onClick: () => void;
}>(({ value, onClick }) => {
  return (
    <button
      type="button"
      className={css`
        ${buttonReset}
        cursor: pointer;
        width: 24px;
        height: 24px;
        position: absolute;
        right: 10px;

        &:focus {
          outline: none;
        }
      `}
      tabIndex={-1}
      onClick={onClick}
      aria-label={value ? 'Hide password' : 'Show password'}
    >
      <div
        className={css`
          display: flex;
          justify-content: center;
          align-items: center;
        `}
      >
        <ShowPasswordIcon
          className={cx(
            !value
              ? css`
                  display: none;
                `
              : css`
                  display: block;

                  button:hover > div > & {
                    display: none;
                  }
                `,
          )}
          role="img"
        />
        <ShowPasswordHoverIcon
          className={cx(
            !value
              ? css`
                  display: none;
                `
              : css`
                  display: none;

                  button:hover > div > & {
                    display: block;
                  }
                `,
          )}
          role="img"
        />
        <HidePasswordIcon
          className={cx(
            value
              ? css`
                  display: none;
                `
              : css`
                  display: block;

                  button:hover > div > & {
                    display: none;
                  }
                `,
          )}
          role="img"
        />
        <HidePasswordHoverIcon
          className={cx(
            value
              ? css`
                  display: none;
                `
              : css`
                  display: none;

                  button:hover > div > & {
                    display: block;
                  }
                `,
          )}
          role="img"
        />
      </div>
    </button>
  );
});
