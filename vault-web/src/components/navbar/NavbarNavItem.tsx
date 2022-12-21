import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { Button } from '@restart/ui';
import { ComponentProps, forwardRef, memo, ReactNode } from 'react';

import {
  buttonHoverTransition,
  buttonReset,
} from '../../styles/mixins/buttons';
import { allStates } from '../../styles/mixins/hover';

export const NavbarNavItem = memo(
  forwardRef<
    HTMLDivElement,
    ComponentProps<typeof Button> & {
      backgroundClassName?: string;
      icon: ReactNode;
    }
  >(({ children, className, backgroundClassName, icon, ...props }, ref) => {
    const theme = useTheme();

    return (
      <Button
        className={cx(
          css`
            ${buttonReset}
            margin: 0;
            padding: 0;

            ${allStates} {
              text-decoration: none;
            }
          `,
          theme.isMobile
            ? css`
                min-width: 37px;
              `
            : css`
                margin-left: 6px;
                margin-right: 6px;
                margin-bottom: 5px;
                min-width: 45px;
              `
        )}
        {...props}
        ref={ref}
      >
        <div
          className={css`
            display: flex;
            flex-direction: column;
            align-items: center;
          `}
        >
          <div
            className={cx(
              css`
                display: flex;
                justify-content: center;
                align-items: center;
                width: 32px;
                height: 32px;
                border-radius: 3px;
                transition: ${buttonHoverTransition};
              `,
              !theme.isMobile &&
                css`
                  margin-bottom: 3px;
                `,
              backgroundClassName
            )}
          >
            {icon}
          </div>
          {!theme.isMobile ? (
            <div
              className={css`
                display: block;
                color: ${theme.colors.text};
                font-size: 11px;
                line-height: 20px;
                font-weight: 600;
                text-align: center;
              `}
            >
              {children}
            </div>
          ) : null}
        </div>
      </Button>
    );
  })
);
