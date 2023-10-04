import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback, useState } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';

import { TransfersList } from './TransfersList';
import { TransfersSummary } from './TransfersSummary';

export const Transfers = memo(() => {
  const theme = useTheme();
  const [areDetailsVisible, setDetailsVisible] = useState(false);
  const toggleDetailsVisible = useCallback(
    () => setDetailsVisible((visible) => !visible),
    [],
  );
  const [isActive] = useSubscribe(
    (v, cb) => v.transfersIsActiveSubscribe(cb),
    (v) => v.transfersIsActiveData,
    [],
  );

  if (!isActive) {
    return null;
  }

  return (
    <div
      className={css`
        display: block;
        position: fixed;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: ${theme.zindex.transfers};
        border-top: 1px solid ${theme.colors.border};
        background-color: #fff;
      `}
    >
      <TransfersSummary
        areDetailsVisible={areDetailsVisible}
        toggleDetailsVisible={toggleDetailsVisible}
      />
      <div
        className={cx(
          css`
            height: 0px;
            transition: height 0.3s ease-out;
            padding: 2px 0 0;
          `,
          areDetailsVisible &&
            css`
              height: 191px;
              overflow-y: auto;
            `,
        )}
      >
        {areDetailsVisible ? <TransfersList /> : null}
      </div>
    </div>
  );
});
