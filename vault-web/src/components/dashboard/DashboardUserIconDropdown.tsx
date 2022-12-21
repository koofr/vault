import { css } from '@emotion/css';
import Dropdown from '@restart/ui/Dropdown';
import { useDropdownToggle } from '@restart/ui/DropdownToggle';
import { memo, useState } from 'react';

import { UserIcon } from '../UserIcon';

import { DashboardMenu } from './DashboardMenu';

export const DashboardUserIcon = memo(() => {
  const [props] = useDropdownToggle();

  return (
    <div
      role="button"
      className={css`
        cursor: pointer;
      `}
      {...props}
    >
      <UserIcon />
    </div>
  );
});

export const DashboardUserIconDropdown = memo(() => {
  const [isVisible, setVisible] = useState(false);

  return (
    <Dropdown
      show={isVisible}
      onToggle={(value) => setVisible(value)}
      placement="bottom-end"
    >
      <DashboardUserIcon />
      <DashboardMenu />
    </Dropdown>
  );
});
