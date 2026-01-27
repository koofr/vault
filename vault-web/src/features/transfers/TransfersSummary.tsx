import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, MouseEvent, useCallback, useState } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

import TransfersFailedIcon from '../../assets/images/transfers-failed.svg?react';
import TransfersIcon from '../../assets/images/transfers.svg?react';
import { Button } from '../../components/Button';
import { Progress } from '../../components/Progress';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

export const TRANSFERS_SUMMARY_HEIGHT = 48;

export const TransfersSummary = memo<{
  areDetailsVisible: boolean;
  toggleDetailsVisible: () => void;
}>(({ areDetailsVisible, toggleDetailsVisible }) => {
  const intl = useIntl();
  const theme = useTheme();
  const webVault = useWebVault();
  const [transfersSummary] = useSubscribe(
    (v, cb) => v.transfersSummarySubscribe(cb),
    (v) => v.transfersSummaryData,
    [],
  );
  const [isSizeVisible, setSizeVisible] = useState(false);
  const [isSpeedVisible, setSpeedVisible] = useState(false);
  const onAbortAllClick = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      event.stopPropagation();
      webVault.transfersAbortAll();
    },
    [webVault],
  );
  const onRetryAllClick = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      event.stopPropagation();
      webVault.transfersRetryAll();
    },
    [webVault],
  );
  const onCountClick = useCallback((event: MouseEvent<HTMLElement>) => {
    event.stopPropagation();
    setSizeVisible((isVisible) => !isVisible);
  }, []);
  const onTimeClick = useCallback((event: MouseEvent<HTMLElement>) => {
    event.stopPropagation();
    setSpeedVisible((isVisible) => !isVisible);
  }, []);

  if (transfersSummary === undefined) {
    return null;
  }

  const {
    totalCount,
    doneCount,
    failedCount,
    sizeProgressDisplay,
    percentage,
    remainingTimeDisplay,
    speedDisplay,
    isTransferring,
    isAllDone,
    canRetryAll,
    canAbortAll,
  } = transfersSummary;

  return (
    <div
      className={cx(
        css`
          height: ${TRANSFERS_SUMMARY_HEIGHT}px;
          cursor: pointer;
          display: flex;
          align-items: center;
        `,
        theme.isMobile
          ? css`
              padding: 0 7px;
            `
          : css`
              padding: 0 25px;
            `,
        areDetailsVisible &&
          css`
            box-shadow: ${theme.boxShadow};
          `,
      )}
      onClick={toggleDetailsVisible}
    >
      <div
        className={cx(
          css`
            flex-grow: 1;
            flex-basis: 0;
          `,
          theme.isMobile
            ? css`
                display: none;
              `
            : css`
                display: flex;
              `,
        )}
      ></div>
      <div
        className={css`
          max-width: 840px;
          margin: auto;
          display: flex;
          align-items: center;
          flex-grow: 3;
          font-size: 13px;
        `}
      >
        <div
          className={cx(
            css`
              display: flex;
              align-items: center;
            `,
            theme.isMobile
              ? css`
                  margin-right: 5px;
                `
              : css`
                  width: 160px;
                `,
          )}
        >
          <div
            className={css`
              width: 32px;
              height: 32px;
              display: flex;
              justify-content: center;
              align-items: center;
              margin-right: 8px;
            `}
          >
            {failedCount > 0 ? (
              <TransfersFailedIcon role="img" />
            ) : (
              <TransfersIcon role="img" />
            )}
          </div>
          <div
            className={css`
              font-size: 13px;
              font-weight: normal;
              color: ${theme.colors.text};
            `}
            onClick={onCountClick}
          >
            {isSizeVisible ? (
              <span
                className={css`
                  font-weight: 600;
                `}
              >
                {sizeProgressDisplay}
              </span>
            ) : (
              <FormattedMessage
                id="web.transfers.summary.progress.text"
                description="Status line in the transfers summary bar showing completed versus total transfer count."
                defaultMessage="<b>{done_count} / {total_count}</b> done"
                values={{
                  done_count: doneCount,
                  total_count: totalCount,
                  b: (chunks) => (
                    <span
                      className={css`
                        font-weight: 600;
                      `}
                    >
                      {chunks}
                    </span>
                  ),
                }}
              />
            )}
          </div>
        </div>
        <div
          className={css`
            flex-grow: 1;
            margin: -4px 0 0;
          `}
        >
          <div
            className={css`
              padding-top: 4px;
              width: 100%;
            `}
          >
            <Progress percentage={percentage} />
          </div>
        </div>
        <div
          className={cx(
            css`
              display: flex;
              justify-content: flex-end;
              align-items: center;
            `,
            theme.isMobile
              ? css`
                  margin-left: 5px;
                `
              : css`
                  width: 160px;
                `,
          )}
        >
          {isTransferring ? (
            <div
              className={css`
                display: flex;
                flex-direction: column;
                font-size: 13px;
                font-weight: normal;
                color: ${theme.colors.text};
              `}
              onClick={onTimeClick}
            >
              {isSpeedVisible ? (
                <span
                  className={css`
                    font-weight: 600;
                  `}
                >
                  {speedDisplay}
                </span>
              ) : (
                <span>
                  <FormattedMessage
                    id="web.transfers.summary.remaining_time.text"
                    description="Status line in the transfers summary bar showing estimated remaining time (e.g. '1 minute remaining')."
                    defaultMessage="<b>{time}</b> remaining"
                    values={{
                      time: remainingTimeDisplay,
                      b: (chunks) => (
                        <span
                          className={css`
                            font-weight: 600;
                          `}
                        >
                          {chunks}
                        </span>
                      ),
                    }}
                  />
                </span>
              )}
            </div>
          ) : null}
          {!isTransferring ? (
            <Button type="button" variant="primary-inline">
              {areDetailsVisible ? (
                <FormattedMessage
                  id="web.transfers.summary.hide_details.button"
                  description="Button label in the transfers summary bar to collapse the transfer details list."
                  defaultMessage="Hide details"
                />
              ) : (
                <FormattedMessage
                  id="web.transfers.summary.show_details.button"
                  description="Button label in the transfers summary bar to expand the transfer details list."
                  defaultMessage="Show details"
                />
              )}
            </Button>
          ) : null}
        </div>
      </div>
      <div
        className={cx(
          css`
            display: flex;
            justify-content: flex-end;
          `,
          theme.isMobile
            ? css`
                margin-left: 5px;
              `
            : css`
                flex-grow: 1;
                flex-basis: 0;
              `,
        )}
      >
        {canRetryAll ? (
          <Button
            type="button"
            variant="primary-inline"
            className={css`
              flex-shrink: 0;
              margin-right: 15px;
            `}
            onClick={onRetryAllClick}
            title={intl.formatMessage({
              id: 'web.transfers.summary.retry_all.tooltip',
              description:
                'Tooltip for the Retry button in the transfers summary bar that retries all failed transfers.',
              defaultMessage: 'Retry failed transfers',
            })}
          >
            <FormattedMessage
              id="web.transfers.summary.retry_all.button"
              description="Button label in the transfers summary bar to retry all failed transfers."
              defaultMessage="Retry"
            />
          </Button>
        ) : null}
        {canAbortAll ? (
          isAllDone ? (
            <Button
              type="button"
              variant="inline"
              className={css`
                flex-shrink: 0;
              `}
              onClick={onAbortAllClick}
              title={intl.formatMessage({
                id: 'web.transfers.summary.clear_all.tooltip',
                description:
                  'Tooltip for the Clear button in the transfers summary bar to clear the completed transfer list.',
                defaultMessage: 'Clear all transfers',
              })}
            >
              <FormattedMessage
                id="web.transfers.summary.clear_all.button"
                description="Button label in the transfers summary bar to clear all finished transfers."
                defaultMessage="Clear"
              />
            </Button>
          ) : (
            <Button
              type="button"
              variant="destructive-inline"
              className={css`
                flex-shrink: 0;
              `}
              onClick={onAbortAllClick}
              title={intl.formatMessage({
                id: 'web.transfers.summary.cancel_all.tooltip',
                description:
                  'Tooltip for the Cancel button in the transfers summary bar to cancel all active transfers.',
                defaultMessage: 'Cancel all transfers',
              })}
            >
              <FormattedMessage
                id="web.transfers.summary.cancel_all.button"
                description="Button label in the transfers summary bar to cancel all active transfers."
                defaultMessage="Cancel"
              />
            </Button>
          )
        ) : null}
      </div>
    </div>
  );
});
TransfersSummary.displayName = 'TransfersSummary';
