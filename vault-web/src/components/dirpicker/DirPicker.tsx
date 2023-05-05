import { css, cx, keyframes } from '@emotion/css';
import { useTheme } from '@emotion/react';
import range from 'lodash/range';
import { memo, useMemo, useRef } from 'react';

import { ReactComponent as LoadingIcon } from '../../assets/images/loading.svg';
import { useSubscribe } from '../../webVault/useSubscribe';

import { useIsMobile } from '../useIsMobile';

import { DirPickerItemIcon } from './DirPickerItemIcon';

const spinner = keyframes`
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(359deg);
  }
`;

export const DirPicker = memo<{
  pickerId: number;
  onClick: (
    pickerId: number,
    itemId: string,
    isArrow: boolean
  ) => Promise<void>;
}>(({ pickerId, onClick }) => {
  const isMobile = useIsMobile();
  const theme = useTheme();
  const [items] = useSubscribe(
    (v, cb) => v.dirPickersItemsSubscribe(pickerId, cb),
    (v) => v.dirPickersItemsData,
    [pickerId]
  );
  const selectedItemId = items.find((item) => item.isSelected)?.id;
  const nextScrollToItemId = useRef<string>();
  useMemo(() => {
    nextScrollToItemId.current = selectedItemId;
  }, [selectedItemId]);

  return (
    <div
      className={
        isMobile
          ? css`
              padding: 9px 0;
            `
          : css`
              width: 510px;
              padding: 9px 0;
            `
      }
    >
      {items.map((item) => (
        <div
          key={item.id}
          className={cx(
            css`
              display: flex;
              cursor: pointer;
              padding-left: 15px;
              height: 32px;
            `,
            item.isSelected &&
              css`
                background-color: #f4f5f5;
              `,
            item.isSelected && 'is-selected'
          )}
          onClick={() => onClick(pickerId, item.id, false)}
          ref={(el) => {
            if (
              el !== null &&
              nextScrollToItemId.current !== undefined &&
              item.id === nextScrollToItemId.current
            ) {
              if ((el as any).scrollIntoViewIfNeeded !== undefined) {
                (el as any).scrollIntoViewIfNeeded();
              } else {
                el.scrollIntoView();
              }
            }
          }}
        >
          {range(0, item.spaces).map((i) => (
            <div
              key={i}
              className={css`
                width: 18px;
                height: 32px;
                margin-right: 8px;
                flex-shrink: 0;
              `}
            ></div>
          ))}
          {item.hasArrow ? (
            <div
              className={css`
                width: 18px;
                height: 32px;
                margin-right: 8px;
                display: flex;
                justify-content: center;
                align-items: center;
                flex-shrink: 0;
              `}
              onClick={(e) => {
                e.stopPropagation();
                onClick(pickerId, item.id, true);
              }}
            >
              {item.isOpen ? (
                <div
                  className={css`
                    display: block;
                    width: 0;
                    height: 0;
                    border-top: 4px solid transparent;
                    border-bottom: 4px solid transparent;
                    border-right: none;
                    border-left: 4px solid ${theme.colors.textLight};
                    margin: 12px 16px 14px 16px;
                  `}
                />
              ) : (
                <div
                  className={css`
                    display: block;
                    width: 0;
                    height: 0;
                    border-top: 4px solid ${theme.colors.textLight};
                    border-bottom: none;
                    border-right: 4px solid transparent;
                    border-left: 4px solid transparent;
                    margin: 14px 16px 14px 12px;
                  `}
                />
              )}
            </div>
          ) : null}
          <div
            className={css`
              width: 18px;
              height: 32px;
              display: flex;
              justify-content: center;
              align-items: center;
              margin-right: 8px;
              flex-shrink: 0;
            `}
          >
            <DirPickerItemIcon
              itemType={item.typ}
              hoverSelector="div:hover > * > &, div.is-selected > * > &"
            />
          </div>
          <div
            title={item.text}
            className={css`
              color: ${theme.colors.textLight};
              font-size: 13px;
              line-height: 32px;
              font-weight: 600;
              text-overflow: ellipsis;
              white-space: nowrap;
              overflow: hidden;

              div:hover > &,
              div.is-selected > & {
                color: ${theme.colors.text};
              }
            `}
          >
            {item.text}
          </div>
          {item.isLoading ? (
            <div
              className={css`
                width: 18px;
                height: 32px;
                display: flex;
                justify-content: center;
                align-items: center;
                margin-left: 4px;
              `}
              aria-label="Loading..."
            >
              <LoadingIcon
                className={css`
                  animation: ${spinner} 2s infinite linear;
                `}
                role="img"
              />
            </div>
          ) : null}
        </div>
      ))}
    </div>
  );
});
