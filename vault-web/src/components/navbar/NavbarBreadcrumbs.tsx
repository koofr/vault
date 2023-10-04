import { css } from '@emotion/css';
import { memo } from 'react';

import { NavbarBreadcrumb, NavbarBreadcrumbInfo } from './NavbarBreadcrumb';
import { NavbarBreadcrumbsSeparator } from './NavbarBreadcrumbsSeparator';

export interface NavbarBreadcrumbsProps {
  breadcrumbs: NavbarBreadcrumbInfo[];
  onClick?: (
    event: React.MouseEvent<any>,
    breadcrumb: NavbarBreadcrumbInfo,
  ) => void;
  onCaretClick?: (
    event: React.MouseEvent<any>,
    breadcrumb: NavbarBreadcrumbInfo,
  ) => void;
}

export const NavbarBreadcrumbs = memo<NavbarBreadcrumbsProps>(
  ({ breadcrumbs, onClick, onCaretClick }) => {
    const breadcrumbsHead = breadcrumbs.slice(0, breadcrumbs.length - 2);
    const breadcrumbsTail = breadcrumbs.slice(
      Math.max(breadcrumbs.length - 2, 0),
    );
    const hasTail = breadcrumbs.length > 2;

    return (
      <div
        className={css`
          margin: 0;
          padding: 0;
          display: flex;
        `}
        aria-label="Breadcrumb"
      >
        {hasTail ? (
          <div
            className={css`
              display: flex;
              flex-grow: 0;
              flex-shrink: 1;
              overflow: hidden;
            `}
          >
            <div
              className={css`
                text-overflow: ellipsis;
                white-space: nowrap;
                overflow: hidden;
              `}
            >
              {breadcrumbsHead.map((breadcrumb, i) => (
                <NavbarBreadcrumb
                  key={breadcrumb.id}
                  breadcrumb={breadcrumb}
                  separator={i < breadcrumbsHead.length - 1}
                  onClick={onClick}
                  onCaretClick={onCaretClick}
                />
              ))}
            </div>
          </div>
        ) : null}
        {hasTail ? (
          <div
            className={css`
              flex-shrink: 0;
            `}
          >
            <NavbarBreadcrumbsSeparator />
          </div>
        ) : null}
        <div
          className={css`
            flex-shrink: 0;
          `}
        >
          {breadcrumbsTail.map((breadcrumb, i) => (
            <NavbarBreadcrumb
              key={breadcrumb.id}
              breadcrumb={breadcrumb}
              separator={i < breadcrumbsTail.length - 1}
              onClick={onClick}
              onCaretClick={onCaretClick}
            />
          ))}
        </div>
      </div>
    );
  },
);
