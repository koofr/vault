import { css } from '@emotion/css';
import { memo } from 'react';
import { useIntl } from 'react-intl';
import { Link, To } from 'react-router-dom';

import NavbarBackIcon from '../../assets/images/navbar-back.svg?react';

export const NavbarBack = memo<{ to: To }>(({ to }) => {
  const intl = useIntl();

  return (
    <Link
      to={to}
      className={css`
        display: flex;
        align-items: center;
      `}
      aria-label={intl.formatMessage({
        id: 'web.navbar_back.aria_label',
        description: 'Accessibility label for the navbar back button.',
        defaultMessage: 'Back',
      })}
    >
      <NavbarBackIcon role="img" />
    </Link>
  );
});
NavbarBack.displayName = 'NavbarBack';
