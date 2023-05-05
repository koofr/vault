import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, MouseEvent, useCallback, useState } from 'react';

import { ReactComponent as UploadsFailedIcon } from '../../assets/images/uploads-failed.svg';
import { ReactComponent as UploadsIcon } from '../../assets/images/uploads.svg';
import { Button } from '../../components/Button';
import { Progress } from '../../components/Progress';
import { sizeDisplay, sizeOfDisplay } from '../../utils/sizeDisplay';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';
import { remainingTimeDisplay } from '../../utils/remainingTime';

export const UPLOADS_SUMMARY_HEIGHT = 48;

export const UploadsSummary = memo<{
  areDetailsVisible: boolean;
  toggleDetailsVisible: () => void;
}>(({ areDetailsVisible, toggleDetailsVisible }) => {
  const theme = useTheme();
  const webVault = useWebVault();
  const [
    {
      totalCount,
      doneCount,
      failedCount,
      totalBytes,
      doneBytes,
      percentage,
      remainingTime,
      bytesPerSecond,
      isUploading,
      canRetry,
      canAbort,
    },
  ] = useSubscribe(
    (v, cb) => v.uploadsSummarySubscribe(cb),
    (v) => v.uploadsSummaryData,
    []
  );
  let [isSizeVisible, setSizeVisible] = useState(false);
  let [isSpeedVisible, setSpeedVisible] = useState(false);
  const onAbortAllClick = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      event.stopPropagation();
      webVault.uploadsAbortAll();
    },
    [webVault]
  );
  const onRetryAllClick = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      event.stopPropagation();
      webVault.uploadsRetryAll();
    },
    [webVault]
  );
  const onCountClick = useCallback((event: MouseEvent<HTMLElement>) => {
    event.stopPropagation();
    setSizeVisible((isVisible) => !isVisible);
  }, []);
  const onTimeClick = useCallback((event: MouseEvent<HTMLElement>) => {
    event.stopPropagation();
    setSpeedVisible((isVisible) => !isVisible);
  }, []);

  return (
    <div
      className={cx(
        css`
          height: ${UPLOADS_SUMMARY_HEIGHT}px;
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
          `
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
              `
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
                `
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
              <UploadsFailedIcon role="img" />
            ) : (
              <UploadsIcon role="img" />
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
                {sizeOfDisplay(doneBytes, totalBytes)}
              </span>
            ) : (
              <span>
                <span
                  className={css`
                    font-weight: 600;
                  `}
                >
                  {doneCount} / {totalCount}
                </span>{' '}
                done
              </span>
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
                `
          )}
        >
          {isUploading ? (
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
                  {sizeDisplay(bytesPerSecond)}/s
                </span>
              ) : (
                <span>
                  <span
                    className={css`
                      font-weight: 600;
                    `}
                  >
                    {remainingTimeDisplay(remainingTime)}
                  </span>{' '}
                  remaining
                </span>
              )}
            </div>
          ) : null}
          {!isUploading ? (
            <Button type="button" variant="primary-inline">
              {areDetailsVisible ? 'Hide details' : 'Show details'}
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
              `
        )}
      >
        {canRetry ? (
          <Button
            type="button"
            variant="primary-inline"
            className={css`
              flex-shrink: 0;
              margin-right: 15px;
            `}
            onClick={onRetryAllClick}
            title="Retry failed uploads"
          >
            Retry
          </Button>
        ) : null}
        {canAbort ? (
          <Button
            type="button"
            variant="destructive-inline"
            className={css`
              flex-shrink: 0;
            `}
            onClick={onAbortAllClick}
            title="Cancel all uploads"
          >
            Cancel
          </Button>
        ) : null}
      </div>
    </div>
  );
});
