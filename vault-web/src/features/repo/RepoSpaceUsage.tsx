import { css } from '@emotion/css';
import { memo, useCallback, useEffect, useMemo } from 'react';

import { Button } from '../../components/Button';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

export const RepoSpaceUsage = memo<{ repoId: string }>(({ repoId }) => {
  const webVault = useWebVault();
  const usageId = useMemo(
    () => webVault.repoSpaceUsageCreate(repoId),
    [webVault, repoId],
  );
  useEffect(() => {
    return () => {
      webVault.repoSpaceUsageDestroy(usageId);
    };
  }, [webVault, usageId]);
  const [info] = useSubscribe(
    (v, cb) => v.repoSpaceUsageInfoSubscribe(usageId, cb),
    (v) => v.repoSpaceUsageInfoData,
    [usageId],
  );
  const onCalculate = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();

      webVault.repoSpaceUsageCalculate(usageId);
    },
    [webVault, usageId],
  );

  if (info === undefined) {
    return null;
  }

  return (
    <div>
      <h2
        className={css`
          font-size: 28px;
          font-weight: normal;
          margin: 0 0 20px;
        `}
      >
        Space used
      </h2>

      {info.status.type === 'Error' ? (
        <div
          className={css`
            background-color: #fbedeb;
            padding: 6px 15px;
            border-radius: 3px;
            margin: 0 0 15px;
          `}
        >
          {info.status.error}
        </div>
      ) : null}

      {info.status.type === 'Initial' || info.status.type === 'Error' ? (
        <Button
          type="button"
          variant="primary"
          onClick={onCalculate}
          className={css`
            height: 36px;
          `}
        >
          Calculate
        </Button>
      ) : null}

      {info.status.type === 'Loading' ? (
        <div
          className={css`
            height: 36px;
          `}
        >
          Loading...
        </div>
      ) : null}

      {info.spaceUsedDisplay !== undefined ? (
        <div
          className={css`
            height: 36px;
          `}
        >
          {info.spaceUsedDisplay}
        </div>
      ) : null}
    </div>
  );
});
