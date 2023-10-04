import { memo, useCallback, useEffect, useMemo } from 'react';

import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useDocumentTitle } from '../../utils/useDocumentTitle';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RepoUnlockForm } from './RepoUnlockForm';

export const RepoUnlock = memo<{ repoId: string }>(({ repoId }) => {
  const webVault = useWebVault();
  let unlockId = useMemo(
    () =>
      webVault.repoUnlockCreate(repoId, {
        mode: 'Unlock',
      }),
    [webVault, repoId],
  );
  useEffect(() => {
    return () => {
      webVault.repoUnlockDestroy(unlockId);
    };
  }, [webVault, unlockId]);
  const [info] = useSubscribe(
    (v, cb) => v.repoUnlockInfoSubscribe(unlockId, cb),
    (v) => v.repoUnlockInfoData,
    [unlockId],
  );
  const onUnlock = useCallback(
    (password: string) => {
      webVault.repoUnlockUnlock(unlockId, password);
    },
    [webVault, unlockId],
  );
  const navbarHeader = useSingleNavbarBreadcrumb(info?.repoName ?? '');
  useDocumentTitle(info?.repoName);

  if (info === undefined) {
    return null;
  }

  return (
    <DashboardLayout navbarHeader={navbarHeader}>
      {info !== undefined ? (
        <RepoUnlockForm info={info} onUnlock={onUnlock} />
      ) : null}
    </DashboardLayout>
  );
});
