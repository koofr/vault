import { css } from '@emotion/css';
import { memo } from 'react';
import { useIntl } from 'react-intl';
import { Link, To } from 'react-router-dom';

import NavbarCloseIcon from '../../assets/images/navbar-close.svg?react';

export const NavbarClose = memo<{ to: To }>(({ to }) => {
  const intl = useIntl();

  return (
    <Link
      to={to}
      className={css`
        display: flex;
        align-items: center;
      `}
      aria-label={intl.formatMessage({
        id: 'web.navbar_close.aria_label',
        description: 'Accessibility label for the navbar close button.',
        defaultMessage: 'Close',
      })}
    >
      <NavbarCloseIcon />
    </Link>
  );
});
NavbarClose.displayName = 'NavbarClose';
