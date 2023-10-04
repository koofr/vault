import { css } from '@emotion/css';
import { memo } from 'react';
import { Link } from 'react-router-dom';

import VaultLogoIcon from '../../assets/images/vault-logo.svg?react';

export const NavbarLogo = memo(() => {
  return (
    <Link
      to="/"
      className={css`
        display: flex;
        align-items: center;
      `}
    >
      <VaultLogoIcon role="img" aria-label="Koofr Vault logo" />
    </Link>
  );
});
