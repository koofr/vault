import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { useDropdownMenu } from '@restart/ui/esm/DropdownMenu';
import { memo, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';

import { ReactComponent as DirPickerItemHostedHoverIcon } from '../../assets/images/dir-picker-item-hosted-hover.svg';
import { ReactComponent as DirPickerItemHostedIcon } from '../../assets/images/dir-picker-item-hosted.svg';
import { ReactComponent as LogoutHoverIcon } from '../../assets/images/logout-hover.svg';
import { ReactComponent as LogoutIcon } from '../../assets/images/logout.svg';
import { allStates } from '../../styles/mixins/hover';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { UserIcon } from '../UserIcon';
import { Menu, MenuBaseItem, MenuDivider, MenuItem } from '../menu/Menu';
import { useMenuUpdate } from '../menu/useMenuUpdate';

export const DashboardMenuUserInfoItem = memo(() => {
  const theme = useTheme();
  const webVault = useWebVault();
  const baseUrl = webVault.configGetBaseUrl();
  const user = useSubscribe(
    (v, cb) => v.userSubscribe(cb),
    (v) => v.userData,
    []
  );

  if (user === undefined) {
    return null;
  }

  return (
    <MenuBaseItem>
      <a
        href={`${baseUrl}/app/admin/preferences`}
        target="_blank"
        rel="noreferrer"
        className={css`
          display: flex;
          align-items: center;
          padding: 5px 12px;

          ${allStates} {
            color: ${theme.colors.text};
            text-decoration: none;
          }
        `}
      >
        <div
          className={css`
            display: flex;
            margin-right: 10px;
          `}
        >
          <UserIcon />
        </div>
        <div
          className={css`
            display: block;
            overflow: hidden;
          `}
        >
          <div
            className={css`
              font-size: 14px;
              font-weight: normal;
              text-overflow: ellipsis;
              white-space: nowrap;
              overflow: hidden;
            `}
          >
            {user.fullName}
          </div>
        </div>
      </a>
    </MenuBaseItem>
  );
});

export const DashboardMenu = memo(() => {
  const theme = useTheme();
  const navigate = useNavigate();
  const webVault = useWebVault();
  const baseUrl = webVault.configGetBaseUrl();
  const [props, { show, popper }] = useDropdownMenu({
    fixed: true,
    offset: [15, 12],
  });
  useMenuUpdate(show, popper);
  const logout = useCallback(() => {
    webVault.logout();
    navigate('/');
  }, [webVault, navigate]);

  return (
    <Menu
      isVisible={show}
      {...props}
      className={css`
        width: 214px;
        z-index: ${theme.zindex.dashboardMenu};
        overflow-y: auto;
      `}
    >
      <DashboardMenuUserInfoItem />
      <MenuDivider />
      <MenuItem
        icon={<DirPickerItemHostedIcon />}
        iconHover={<DirPickerItemHostedHoverIcon />}
        textClassName={css`
          font-weight: normal;
        `}
        href={baseUrl}
        target="_blank"
        rel="noreferrer"
      >
        Open Koofr
      </MenuItem>
      <MenuDivider />
      <MenuItem
        icon={<LogoutIcon />}
        iconHover={<LogoutHoverIcon />}
        textClassName={css`
          font-weight: normal;
        `}
        onClick={logout}
      >
        Logout
      </MenuItem>
    </Menu>
  );
});
