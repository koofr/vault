import { css, cx } from '@emotion/css';
import styled from '@emotion/styled';
import { Button } from '@restart/ui';
import React from 'react';

import { ToolbarItem, ToolbarItemProps } from '../toolbar/Toolbar';

export const NavbarNavToolbar = styled.nav`
  display: flex;
  align-items: center;
  margin-right: -8px;
`;

export function NavbarNavToolbarItem<
  T extends React.ElementType = typeof Button
>({ ...props }: ToolbarItemProps<T>) {
  return (
    <ToolbarItem
      {...props}
      textClassName={cx(
        css`
          font-size: 12px;
        `,
        props.textClassName
      )}
      className={cx(
        css`
          margin: 0;
        `,
        props.className
      )}
    />
  );
}
