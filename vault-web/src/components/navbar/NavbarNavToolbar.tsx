import { css, cx } from '@emotion/css';
import styled from '@emotion/styled';
import { Button } from '@restart/ui';
import React from 'react';

import { ToolbarItem, ToolbarItemProps } from '../toolbar/Toolbar';

export const NavbarNavToolbar = styled.nav`
  display: flex;
  align-items: center;
`;

export function NavbarNavToolbarItem<
  T extends React.ElementType = typeof Button,
>({ ...props }: ToolbarItemProps<T>) {
  return (
    <ToolbarItem
      {...props}
      textClassName={cx(
        css`
          font-size: 12px;
        `,
        props.textClassName,
      )}
      className={cx(
        css`
          margin: 0;

          &:last-of-type {
            padding: 0;
          }
        `,
        props.className,
      )}
    />
  );
}
