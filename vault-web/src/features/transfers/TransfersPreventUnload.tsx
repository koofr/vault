import { memo } from 'react';

import { usePreventUnload } from '../../utils/usePreventUnload';
import { useSubscribe } from '../../webVault/useSubscribe';

export const TransfersPreventUnload = memo(() => {
  const [isActive] = useSubscribe(
    (v, cb) => v.transfersIsActiveSubscribe(cb),
    (v) => v.transfersIsActiveData,
    [],
  );
  usePreventUnload(isActive);

  return null;
});
