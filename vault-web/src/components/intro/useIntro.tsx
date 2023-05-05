import { useEffect } from 'react';

import { useModal } from '../../utils/useModal';
import { useSubscribe } from '../../webVault/useSubscribe';

const introSeenKey = 'vaultIntroSeen';

export const useIntro = (): {
  isVisible: boolean;
  show: () => void;
  hide: () => void;
} => {
  const introModal = useModal();
  const introModalShow = introModal.show;
  const [repos] = useSubscribe(
    (v, cb) => v.reposSubscribe(cb),
    (v) => v.reposData,
    []
  );

  useEffect(() => {
    if (repos.status.type === 'Loaded') {
      if (repos.repos.length === 0) {
        try {
          if (localStorage.getItem(introSeenKey) !== 'true') {
            introModalShow();

            localStorage.setItem(introSeenKey, 'true');
          }
        } catch {}
      }
    }
  }, [repos, introModalShow]);

  return introModal;
};
