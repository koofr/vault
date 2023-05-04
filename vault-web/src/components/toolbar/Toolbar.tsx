import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import Button from '@restart/ui/Button';
import { memo, ReactNode } from 'react';

import { buttonReset } from '../../styles/mixins/buttons';

export const Toolbar = styled.nav`
  display: flex;
  align-items: center;
  margin: 0 -12px 0 auto;
`;

interface ToolbarItemBaseProps<T extends React.ElementType> {
  as?: T;
  icon?: ReactNode;
  iconHover?: ReactNode;
  textClassName?: string;
}

export type ToolbarItemProps<T extends React.ElementType = typeof Button> =
  ToolbarItemBaseProps<T> &
    Omit<React.ComponentPropsWithoutRef<T>, keyof ToolbarItemBaseProps<T>>;

export function ToolbarItem<T extends React.ElementType = typeof Button>({
  as,
  icon,
  iconHover,
  className,
  textClassName,
  children,
  ...props
}: ToolbarItemProps<T>) {
  const RootComponent = as || Button;
  const theme = useTheme();

  return (
    <RootComponent
      className={cx(
        css`
          ${buttonReset}
          margin: 0 4px 0 12px;
          padding: 0 8px 0 0;
          text-align: left;

          &[disabled] {
            cursor: not-allowed;
            pointer-events: none;
            opacity: 0.7;
          }
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
    </RootComponent>
  );
}

export const ToolbarCancelItem = memo<React.ComponentProps<typeof ToolbarItem>>(
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
