import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import SpaceUsageIcon from '../../assets/images/space-usage.svg?react';
import { Progress } from '../../components/Progress';
import { useIsMobile } from '../../components/useIsMobile';
import { useSubscribe } from '../../webVault/useSubscribe';

import { TRANSFERS_SUMMARY_HEIGHT } from '../transfers/TransfersSummary';

export const SpaceUsage = memo(() => {
  const isMobile = useIsMobile();
  const theme = useTheme();
  const [spaceUsage] = useSubscribe(
    (v, cb) => v.spaceUsageSubscribe(cb),
    (v) => v.spaceUsageData,
    [],
  );
  const [transfersIsActive] = useSubscribe(
    (v, cb) => v.transfersIsActiveSubscribe(cb),
    (v) => v.transfersIsActiveData,
    [],
  );
  const bottom = transfersIsActive ? TRANSFERS_SUMMARY_HEIGHT : 0;

  if (spaceUsage === undefined) {
    return null;
  }

  return (
    <div
      className={cx(
        css`
          position: fixed;
          left: 0;
          bottom: 0;
          display: flex;
          flex-direction: column;
          box-shadow: 1px -1px 3px 0 rgb(212, 214, 215);
          background-color: #fff;
          padding: 9px 15px 9px 25px;
          z-index: ${theme.zindex.spaceUsage};
        `,
        isMobile
          ? css`
              width: 100%;
            `
          : css`
              width: 225px;
            `,
      )}
      style={{ bottom: `${bottom}px` }}
    >
      <div
        className={css`
          display: flex;
          flex-direction: row;
          align-items: center;
        `}
      >
        <div
          className={css`
            width: 32px;
            height: 32px;
            display: flex;
            justify-content: center;
            align-items: center;
            flex-shrink: 0;
            margin-right: 7px;
          `}
        >
          <SpaceUsageIcon role="img" />
        </div>
        <div
          className={css`
            display: flex;
            flex-direction: column;
            flex-grow: 1;
          `}
        >
          <div
            className={css`
              font-size: 11px;
              font-weight: normal;
              color: ${theme.colors.textLight};
              margin-bottom: 5px;
            `}
          >
            {spaceUsage.usedDisplay} of {spaceUsage.totalDisplay} used
          </div>
          <div
            className={css`
              margin-bottom: 2px;
            `}
          >
            <Progress
              percentage={spaceUsage.percentage}
              severity={spaceUsage.severity}
            />
          </div>
        </div>
      </div>
    </div>
  );
});
