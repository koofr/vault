import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import Dropdown from '@restart/ui/Dropdown';
import { useDropdownToggle } from '@restart/ui/DropdownToggle';
import { memo, useState } from 'react';

import AddInverseIcon from '../../assets/images/add-inverse.svg?react';
import { NavbarNav } from '../../components/navbar/NavbarNav';
import { NavbarNavItem } from '../../components/navbar/NavbarNavItem';

import { RepoFilesAddMenu } from './RepoFilesAddMenu';

export const AddButton = memo(() => {
  const theme = useTheme();
  const [props] = useDropdownToggle();

  return (
    <NavbarNavItem
      backgroundClassName={css`
        background-color: ${theme.colors.primary};

        &:hover {
          background-color: ${theme.colors.primaryHover};
        }
      `}
      icon={<AddInverseIcon role="img" />}
      {...props}
    >
      Add
    </NavbarNavItem>
  );
});

export const AddButtonDropdown = memo(() => {
  const [isVisible, setVisible] = useState(false);

  return (
    <Dropdown
      show={isVisible}
      onToggle={(value) => setVisible(value)}
      placement="bottom"
    >
      <AddButton />
      <RepoFilesAddMenu />
    </Dropdown>
  );
});

export const RepoFilesNav = memo(() => {
  return (
    <NavbarNav>
      <AddButtonDropdown />
    </NavbarNav>
  );
});
