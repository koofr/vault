import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import Button, { ButtonProps } from '@restart/ui/Button';
import { forwardRef, memo, PropsWithChildren, ReactNode } from 'react';

import { buttonReset } from '../../styles/mixins/buttons';
import { allStates } from '../../styles/mixins/hover';

export const MenuBaseItem = styled.div`
  margin: 0;
  padding: 0;
  width: 100%;
  text-overflow: ellipsis;
  white-space: nowrap;
  overflow: hidden;

  &:hover {
    background-color: ${({ theme }) => theme.colors.hover};
  }
`;

export const MenuDivider = styled.div`
  padding: 0;
  margin: 6px 0;
  height: 1px;
  background-color: ${({ theme }) => theme.colors.borderLight};
`;

export const MenuItem = memo<
  PropsWithChildren<
    {
      icon?: ReactNode;
      iconHover?: ReactNode;
      textClassName?: string;
    } & ButtonProps
  >
>(({ icon, iconHover, textClassName, children, ...props }) => {
  const theme = useTheme();

  const buttonContentEl = (
    <div
      className={css`
        display: flex;
        align-items: center;
        width: 100%;
        height: 32px;
      `}
    >
      {icon !== undefined ? (
        <div
          className={cx(
            css`
              display: flex;
              justify-content: center;
              align-items: center;
              margin-right: 8px;
              width: 32px;
              height: 32px;
            `,
            iconHover !== undefined &&
              css`
                div:hover > * > * > & {
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
            margin-right: 8px;
            width: 32px;
            height: 32px;

            div:hover > * > * > & {
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
            flex-grow: 1;
            font-size: 14px;
            font-weight: 600;
            color: ${theme.colors.text};
            text-overflow: ellipsis;
            white-space: nowrap;
            overflow: hidden;
          `,
          textClassName
        )}
      >
        {children}
      </div>
    </div>
  );

  const buttonClassName = cx(
    css`
      ${buttonReset}
      display: block;
      text-align: left;
      width: 100%;
      padding: 0 10px 0 25px;

      ${allStates} {
        text-decoration: none;
      }
    `,
    icon !== undefined &&
      css`
        padding-left: 10px;
      `
  );

  const buttonEl = (
    <Button className={buttonClassName} {...props}>
      {buttonContentEl}
    </Button>
  );

  return <MenuBaseItem>{buttonEl}</MenuBaseItem>;
});

export const Menu = memo(
  forwardRef<
    HTMLDivElement,
    PropsWithChildren<{
      isVisible: boolean;
      className?: string;
      style?: React.CSSProperties;
      'aria-labelledby'?: string;
    }>
  >(({ isVisible, className, children, ...props }, ref) => {
    const theme = useTheme();

    return (
      <div
        ref={ref}
        className={cx(
          css`
            flex-direction: column;
            width: 189px;
            border-radius: 3px;
            background-color: #fff;
            box-shadow: ${theme.boxShadow};
            border: 1px solid ${theme.colors.border};
            margin: 0;
            padding: 5px 0;
          `,
          isVisible
            ? css`
                display: flex;
              `
            : css`
                display: none;
              `,
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  })
);
