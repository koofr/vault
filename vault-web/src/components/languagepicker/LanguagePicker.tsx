import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { useDropdownToggle } from '@restart/ui/DropdownToggle';
import { memo } from 'react';

import LanguagePickerHoverIcon from '../../assets/images/language-picker-hover.svg?react';
import LanguagePickerIcon from '../../assets/images/language-picker.svg?react';
import { useIntlLocales } from '../../features/intl/IntlLocalesContext';

export const LanguagePicker = memo(
  ({
    size,
    dropdownToggleClassName,
  }: {
    size: 'small' | 'large';
    dropdownToggleClassName?: string;
  }) => {
    const [props] = useDropdownToggle();
    const theme = useTheme();
    const { currentLocale } = useIntlLocales();

    if (currentLocale === undefined) {
      return null;
    }

    return (
      <div
        className={css`
          display: flex;
          flex-direction: column;
          align-items: center;
        `}
      >
        <div
          {...props}
          className={cx(
            css`
              display: flex;
              flex-direction: row;
              align-items: center;
              cursor: pointer;
            `,
            dropdownToggleClassName,
          )}
        >
          <div
            className={css`
              display: flex;
              width: 32px;
              height: 32px;
              padding: 7px 7px;
              flex-shrink: 0;
            `}
          >
            <LanguagePickerIcon
              className={css`
                div:hover > & {
                  display: none;
                }
              `}
              role="img"
            />
            <LanguagePickerHoverIcon
              className={css`
                display: none;

                div:hover > & {
                  display: inline;
                }
              `}
              role="img"
            />
          </div>
          <div
            className={cx(
              css`
                font-weight: normal;
                color: ${theme.colors.text};
                text-overflow: ellipsis;
                white-space: nowrap;
                overflow: hidden;
              `,
              size === 'small' &&
                css`
                  font-size: 12px;
                `,
              size === 'large' &&
                css`
                  font-size: 16px;
                `,
            )}
          >
            {currentLocale.name}
          </div>
        </div>
      </div>
    );
  },
);
LanguagePicker.displayName = 'LanguagePicker';
