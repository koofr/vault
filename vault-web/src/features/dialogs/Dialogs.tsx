import { memo } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';

import { Dialog } from './Dialog';

export const Dialogs = memo<{}>(() => {
  const [dialogIds] = useSubscribe(
    (v, cb) => v.dialogsSubscribe(cb),
    (v) => v.dialogsData,
    [],
  );

  if (dialogIds === undefined || dialogIds.length === 0) {
    return null;
  }

  return (
    <>
      {dialogIds.map((dialogId) => (
        <Dialog key={dialogId} dialogId={dialogId} />
      ))}
    </>
  );
});
