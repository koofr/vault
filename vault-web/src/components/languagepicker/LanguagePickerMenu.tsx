import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { useDropdownMenu } from '@restart/ui/DropdownMenu';
import { memo } from 'react';

import { useIntlLocales } from '../../features/intl/IntlLocalesContext';

import { Menu, MenuItem } from '../menu/Menu';
import { useMenuUpdate } from '../menu/useMenuUpdate';

export const LanguagePickerMenu = memo(() => {
  const theme = useTheme();
  const [props, { show, popper, toggle }] = useDropdownMenu();
  useMenuUpdate(show, popper);
  const { locales, changeLocale } = useIntlLocales();

  if (locales === undefined) {
    return null;
  }

  return (
    <Menu
      isVisible={show}
      {...props}
      className={css`
        width: 214px;
        z-index: ${theme.zindex.languagePickerMenu};
        overflow-y: auto;
      `}
    >
      {locales.map((locale) => (
        <MenuItem
          key={locale.locale}
          onClick={(event) => {
            toggle?.(false, event);
            changeLocale(locale.locale);
          }}
        >
          {locale.name}
        </MenuItem>
      ))}
    </Menu>
  );
});
LanguagePickerMenu.displayName = 'LanguagePickerMenu';
