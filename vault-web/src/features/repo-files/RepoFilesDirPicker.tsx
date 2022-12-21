import { useCallback } from 'react';
import { memo } from 'react';

import { DirPicker } from '../../components/dirpicker/DirPicker';
import { useWebVault } from '../../webVault/useWebVault';

export const RepoFilesDirPicker = memo<{
  dirPickerId: number;
}>(({ dirPickerId }) => {
  const webVault = useWebVault();
  const onClick = useCallback(
    (pickerId: number, itemId: string, isArrow: boolean) =>
      webVault.repoFilesDirPickersClick(pickerId, itemId, isArrow),
    [webVault]
  );

  return <DirPicker pickerId={dirPickerId} onClick={onClick} />;
});
