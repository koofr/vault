import { css } from '@emotion/css';
import Dropdown from '@restart/ui/Dropdown';
import { useDropdownToggle } from '@restart/ui/DropdownToggle';
import { memo, useState } from 'react';
import { useIntl } from 'react-intl';

import { UserIcon } from '../../features/user/UserIcon';
import { DashboardMenu } from './DashboardMenu';

export const DashboardUserIcon = memo(() => {
  const intl = useIntl();
  const [props] = useDropdownToggle();

  return (
    <div
      role="button"
      className={css`
        cursor: pointer;
      `}
      aria-label={intl.formatMessage({
        id: 'web.dashboard_user_icon.aria_label',
        description: 'Accessibility label for the user account menu button.',
        defaultMessage: 'User menu',
      })}
      {...props}
    >
      <UserIcon />
    </div>
  );
});
DashboardUserIcon.displayName = 'DashboardUserIcon';

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
DashboardUserIconDropdown.displayName = 'DashboardUserIconDropdown';
