import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import Button, { ButtonProps } from '@restart/ui/Button';
import { memo, ReactNode } from 'react';

import { buttonReset } from '../../styles/mixins/buttons';

export const Toolbar = styled.nav`
  display: flex;
  align-items: center;
  margin: 0 -12px 0 auto;
`;

export type ToolbarItemProps = {
  icon?: ReactNode;
  iconHover?: ReactNode;
  textClassName?: string;
} & ButtonProps;

export const ToolbarItem = memo<ToolbarItemProps>(
  ({ icon, iconHover, children, className, textClassName, ...props }) => {
    const theme = useTheme();

    return (
      <Button
        className={cx(
          css`
            ${buttonReset}
            margin: 0 4px 0 12px;
            padding: 0 8px 0 0;
            text-align: left;
          `,
          className
        )}
        {...props}
      >
        <div
          className={css`
            display: flex;
            align-items: center;
          `}
        >
          {icon !== undefined ? (
            <div
              className={cx(
                css`
                  display: flex;
                  justify-content: center;
                  align-items: center;
                  width: 32px;
                  height: 32px;
                `,
                iconHover !== undefined &&
                  css`
                    *:hover > * > & {
                      display: none;
                    }
                  `
              )}
            >
              {icon}
            </div>
          ) : null}
          {iconHover !== undefined ? (
            <div
              className={css`
                display: none;
                justify-content: center;
                align-items: center;
                width: 32px;
                height: 32px;

                *:hover > * > & {
                  display: flex;
                }
              `}
            >
              {iconHover}
            </div>
          ) : null}
          <div
            className={cx(
              css`
                font-size: 13px;
                font-weight: 600;
                color: ${theme.colors.text};
              `,
              textClassName
            )}
          >
            {children}
          </div>
        </div>
      </Button>
    );
  }
);

export const ToolbarCancelItem = memo<Omit<ToolbarItemProps, 'children'>>(
  ({ textClassName, ...props }) => {
    const theme = useTheme();

    return (
      <ToolbarItem
        textClassName={cx(
          css`
            color: ${theme.colors.destructive};
          `,
          textClassName
        )}
        {...props}
      >
        Cancel
      </ToolbarItem>
    );
  }
);
