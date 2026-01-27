import { css } from '@emotion/css';
import { memo } from 'react';
import { useIntl } from 'react-intl';
import { Link } from 'react-router-dom';

import VaultLogoIcon from '../../assets/images/vault-logo.svg?react';

export const NavbarLogo = memo(() => {
  const intl = useIntl();

  return (
    <Link
      to="/"
      className={css`
        display: flex;
        align-items: center;
      `}
    >
      <VaultLogoIcon
        role="img"
        aria-label={intl.formatMessage({
          id: 'web.navbar_logo.aria_label',
          description:
            'Accessibility label for the Koofr Vault logo in the navbar.',
          defaultMessage: 'Koofr Vault logo',
        })}
      />
    </Link>
  );
});
NavbarLogo.displayName = 'NavbarLogo';
