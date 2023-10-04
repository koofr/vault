import { css } from '@emotion/css';
import { memo } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';

import { TransfersListTransfer } from './TransfersListTransfer';

export const TransfersList = memo(() => {
  const [transfers] = useSubscribe(
    (v, cb) => v.transfersListSubscribe(cb),
    (v) => v.transfersListData,
    [],
  );

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
      `}
    >
      {transfers.transfers.map((transfer) => (
        <TransfersListTransfer key={transfer.id} transfer={transfer} />
      ))}
    </div>
  );
});
