import { css } from '@emotion/css';
import { memo } from 'react';
import { Link, To } from 'react-router-dom';

import NavbarCloseIcon from '../../assets/images/navbar-close.svg?react';

export const NavbarClose = memo<{ to: To }>(({ to }) => {
  return (
    <Link
      to={to}
      className={css`
        display: flex;
        align-items: center;
      `}
      aria-label="Close"
    >
      <NavbarCloseIcon />
    </Link>
  );
});
