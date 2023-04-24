import { css } from '@emotion/css';
import { memo } from 'react';
import { Link, To } from 'react-router-dom';

import { ReactComponent as NavbarBackIcon } from '../../assets/images/navbar-back.svg';

export const NavbarBack = memo<{ to: To }>(({ to }) => {
  return (
    <Link
      to={to}
      className={css`
        display: flex;
        align-items: center;
      `}
      aria-label="Back"
    >
      <NavbarBackIcon role="img" />
    </Link>
  );
});
