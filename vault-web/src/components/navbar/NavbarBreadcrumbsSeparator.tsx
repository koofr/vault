import { css } from '@emotion/css';
import { memo } from 'react';

import { ReactComponent as BreadcrumbsSeparatorIcon } from '../../assets/images/breadcrumbs-separator.svg';

export const NavbarBreadcrumbsSeparator = memo(() => (
  <div
    className={css`
      display: inline;
      flex-grow: 0;
      flex-shrink: 0;
      width: 10px;
      height: 10px;
      margin: 5px 4px 0;
    `}
  >
    <BreadcrumbsSeparatorIcon />
  </div>
));
