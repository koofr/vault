import { css, cx } from '@emotion/css';
import { memo } from 'react';

import { ReactComponent as HidePasswordHoverIcon } from '../assets/images/hide-password-hover.svg';
import { ReactComponent as HidePasswordIcon } from '../assets/images/hide-password.svg';
import { ReactComponent as ShowPasswordHoverIcon } from '../assets/images/show-password-hover.svg';
import { ReactComponent as ShowPasswordIcon } from '../assets/images/show-password.svg';
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
                `
          )}
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
                `
          )}
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
                `
          )}
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
                `
          )}
        />
      </div>
    </button>
  );
});
