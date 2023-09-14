import { css, cx } from '@emotion/css';
import { Theme, useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import RestartUIButton from '@restart/ui/Button';
import React, { memo } from 'react';
import { Link } from 'react-router-dom';

import {
  buttonHoverTransition,
  buttonInlineStyle,
  buttonReset,
  buttonStyle,
} from '../styles/mixins/buttons';

export const BaseButton = styled(RestartUIButton)`
  ${buttonReset}
  border-width: 1px;
  border-style: solid;
  border-radius: 3px;
  text-align: center;
  display: inline-block;
  font-weight: 600;
  font-size: 13px;
  padding: 8px 15px 8px;
  line-height: ${(props) => props.theme.lineHeightBase};
  flex-shrink: 0;

  &:hover,
  &:focus {
    transition: ${buttonHoverTransition};
  }
`;

export const BaseLinkButton = BaseButton.withComponent(Link);

export const BaseAnchorButton = BaseButton.withComponent('a');

export type ButtonVariant =
  | 'primary'
  | 'primary-inline'
  | 'destructive'
  | 'destructive-inline'
  | 'disabled'
  | 'disabled-inline';

export function buttonVariantStyle(
  variant: ButtonVariant | undefined,
  theme: Theme
): string | undefined {
  switch (variant) {
    case 'primary':
      return buttonStyle(
        theme.colors.primary,
        theme.colors.primary,
        theme.colors.textInverse,
        theme.colors.primaryHover,
        theme.colors.primaryHover,
        theme.colors.textInverse
      );
    case 'primary-inline':
      return buttonInlineStyle(theme.colors.primary, theme.colors.primaryHover);
    case 'destructive':
      return buttonStyle(
        theme.colors.destructive,
        theme.colors.destructive,
        theme.colors.textInverse,
        theme.colors.destructiveHover,
        theme.colors.destructiveHover,
        theme.colors.textInverse
      );
    case 'destructive-inline':
      return buttonInlineStyle(
        theme.colors.destructive,
        theme.colors.destructiveHover
      );
    case 'disabled':
      return cx(
        buttonStyle(
          theme.colors.disabled,
          theme.colors.disabled,
          theme.colors.textInverse,
          theme.colors.disabled,
          theme.colors.disabled,
          theme.colors.textInverse
        ),
        css`
          cursor: default;
        `
      );
    case 'disabled-inline':
      return buttonInlineStyle(theme.colors.disabled, theme.colors.disabled);
    default:
      return buttonStyle(
        'transparent',
        theme.colors.borderDarker,
        theme.colors.text,
        'transparent',
        theme.colors.text,
        theme.colors.text
      );
  }
}

export const Button = memo<
  {
    variant?: ButtonVariant;
  } & React.ComponentProps<typeof BaseButton>
>(({ variant, className, ...props }) => {
  const theme = useTheme();

  return (
    <BaseButton
      className={cx(buttonVariantStyle(variant, theme), className)}
      {...props}
    />
  );
});

export const LinkButton = memo<
  {
    variant?: ButtonVariant;
  } & React.ComponentProps<typeof BaseLinkButton>
>(({ variant, className, ...props }) => {
  const theme = useTheme();

  return (
    <BaseLinkButton
      className={cx(buttonVariantStyle(variant, theme), className)}
      {...props}
    />
  );
});
