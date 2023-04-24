import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback } from 'react';
import { Link, To } from 'react-router-dom';

import { allStates } from '../../styles/mixins/hover';

import { NavbarBreadcrumbsSeparator } from './NavbarBreadcrumbsSeparator';

export interface NavbarBreadcrumbInfo {
  id: string;
  name: string;
  link?: To;
  isClickable: boolean;
  hasCaret: boolean;
  isLast: boolean;
}

export const NavbarBreadcrumb = memo<{
  breadcrumb: NavbarBreadcrumbInfo;
  separator: boolean;
  onClick?: (
    event: React.MouseEvent<any>,
    breadcrumb: NavbarBreadcrumbInfo
  ) => void;
  onCaretClick?: (
    event: React.MouseEvent<any>,
    breadcrumb: NavbarBreadcrumbInfo
  ) => void;
}>(
  ({
    breadcrumb,
    separator,
    onClick: onClickFn,
    onCaretClick: onCaretClickFn,
  }) => {
    const { name, link, isClickable, hasCaret, isLast } = breadcrumb;
    const theme = useTheme();
    const onClick = useCallback(
      (event: React.MouseEvent<any>) => {
        if (link === undefined) {
          event.preventDefault();
        }

        if (onClickFn !== undefined && isClickable) {
          onClickFn(event, breadcrumb);
        }
      },
      [link, onClickFn, isClickable, breadcrumb]
    );
    const onCaretClick = useCallback(
      (event: React.MouseEvent<any>) => {
        if (onCaretClickFn !== undefined) {
          onCaretClickFn(event, breadcrumb);
        }
      },
      [onCaretClickFn, breadcrumb]
    );
    const linkClassName = css`
      text-overflow: ellipsis;
      white-space: nowrap;
      overflow: hidden;

      ${allStates} {
        color: ${theme.colors.text};
        text-decoration: none;
      }

      &:hover {
        text-decoration: underline;
      }
    `;

    return (
      <>
        <div
          className={cx(
            css`
              display: inline;
              font-size: 14px;
              font-weight: normal;
            `,
            isLast &&
              css`
                font-weight: 600;
              `
          )}
        >
          {link !== undefined ? (
            <Link
              to={link}
              className={linkClassName}
              onClick={onClick}
              aria-current={isLast ? 'page' : undefined}
            >
              {name}
            </Link>
          ) : isClickable ? (
            <span
              className={linkClassName}
              role="button"
              onClick={onClick}
              aria-current={isLast ? 'page' : undefined}
            >
              {name}
            </span>
          ) : (
            <span onClick={onClick}>{name}</span>
          )}
          {hasCaret && onCaretClick !== undefined ? (
            <div
              className={css`
                display: inline-block;
                width: 10px;
                height: 10px;
                padding: 8px 10px 8px 1px;
                margin-left: 7px;
                cursor: pointer;

                &:after {
                  content: '';
                  display: block;
                  width: 0;
                  height: 0;
                  border-top: 4px solid ${theme.colors.textLight};
                  border-top-color: ${theme.colors.textLight};
                  border-right: 4px solid transparent;
                  border-left: 4px solid transparent;
                }
              `}
              onClick={onCaretClick}
            />
          ) : null}
        </div>
        {separator ? <NavbarBreadcrumbsSeparator /> : null}
      </>
    );
  }
);
