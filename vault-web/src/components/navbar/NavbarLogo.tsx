import { css } from '@emotion/css';
import { memo } from 'react';
import { Link } from 'react-router-dom';

import { ReactComponent as VaultLogoIcon } from '../../assets/images/vault-logo.svg';

export const NavbarLogo = memo(() => {
  return (
    <Link
      to="/"
      className={css`
        display: flex;
        align-items: center;
      `}
    >
      <VaultLogoIcon />
    </Link>
  );
});
