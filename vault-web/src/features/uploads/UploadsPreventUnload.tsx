import { memo } from 'react';

import { usePreventUnload } from '../../utils/usePreventUnload';
import { useSubscribe } from '../../webVault/useSubscribe';

export const UploadsPreventUnload = memo(() => {
  const [isActive] = useSubscribe(
    (v, cb) => v.uploadsIsActiveSubscribe(cb),
    (v) => v.uploadsIsActiveData,
    []
  );
  usePreventUnload(isActive);

  return null;
});
