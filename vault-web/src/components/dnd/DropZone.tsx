import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { forwardRef, memo } from 'react';

export interface DropZoneProps {
  isActive: boolean;
  isOver: boolean;
  isAllowed: boolean;
}

export const DropZone = memo(
  forwardRef<HTMLDivElement, DropZoneProps>(
    ({ isActive, isAllowed, isOver }, ref) => {
      const theme = useTheme();

      const lineCss = cx(
        css`
          position: fixed;
          display: none;
          z-index: ${theme.zindex.dropZoneLines};
          background-color: ${theme.colors.border};
        `,
        isActive &&
          css`
            display: block;
          `,
        isOver &&
          (isAllowed
            ? css`
                background-color: ${theme.colors.successful};
              `
            : css`
                background-color: ${theme.colors.destructive};
              `)
      );

      return (
        <div
          ref={ref}
          className={css`
            display: flex;
            flex-direction: column;
            flex-grow: 1;
          `}
        >
          <div
            className={cx(
              lineCss,
              css`
                top: 0;
                left: 5px;
                right: 0;
                height: 5px;
              `
            )}
          />
          <div
            className={cx(
              lineCss,
              css`
                top: 5px;
                bottom: 0;
                right: 0;
                width: 5px;
              `
            )}
          />
          <div
            className={cx(
              lineCss,
              css`
                bottom: 0;
                left: 0;
                right: 5px;
                height: 5px;
              `
            )}
          />
          <div
            className={cx(
              lineCss,
              css`
                top: 0;
                bottom: 5px;
                left: 0;
                width: 5px;
              `
            )}
          />
        </div>
      );
    }
  )
);
