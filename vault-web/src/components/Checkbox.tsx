import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, MouseEvent } from 'react';

import CheckboxCheck from '../assets/images/checkbox-check.svg?react';
import CheckboxIndeterminate from '../assets/images/checkbox-indeterminate.svg?react';
import { buttonReset } from '../styles/mixins/buttons';

export type CheckboxValue = 'unchecked' | 'checked' | 'indeterminate';

export const Checkbox = memo<{
  value: CheckboxValue;
  small?: boolean;
  onClick?: (event: MouseEvent<HTMLButtonElement>) => void;
}>(({ value, small, onClick }) => {
  const theme = useTheme();

  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={
        value === 'checked'
          ? 'true'
          : value === 'indeterminate'
            ? 'mixed'
            : 'false'
      }
      className={cx(
        css`
          ${buttonReset}
          width: ${small === true ? '15px' : '32px'};
          height: ${small === true ? '15px' : '32px'};
          transition: color;
        `,
        value === 'checked'
          ? css`
              color: #566bb8;
            `
          : css`
              color: ${theme.colors.textLight};

              &:hover {
                color: ${theme.colors.text};
              }
            `,
      )}
      onClick={onClick}
    >
      <div
        className={css`
          display: flex;
          justify-content: center;
          align-items: center;
        `}
      >
        <div
          className={css`
            width: 15px;
            height: 15px;
            border: 1.5px solid currentColor;
            border-radius: 3px;
            position: relative;

            button:focus > div > & {
              outline: 0;
              box-shadow: 0 0 2px 1px rgb(13 110 253 / 25%);
            }
          `}
        >
          <div
            className={cx(
              css`
                display: flex;
                justify-content: center;
                align-items: center;
                position: absolute;
                left: 0;
                top: 0;
                right: 0;
                bottom: 0;
                transition: opacity 0.2s ease-out;
                opacity: 0;
              `,
              value === 'checked' &&
                css`
                  opacity: 1;
                `,
            )}
          >
            <CheckboxCheck />
          </div>
          <div
            className={cx(
              css`
                display: flex;
                justify-content: center;
                align-items: center;
                position: absolute;
                left: 0;
                top: 0;
                right: 0;
                bottom: 0;
                transition: opacity 0.2s ease-out;
                opacity: 0;
              `,
              value === 'indeterminate' &&
                css`
                  opacity: 1;
                `,
            )}
          >
            <CheckboxIndeterminate />
          </div>
        </div>
      </div>
    </button>
  );
});
