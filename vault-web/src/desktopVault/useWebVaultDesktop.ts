import { useWebVault } from '../webVault/useWebVault';

import { WebVaultDesktop } from './WebVaultDesktop';

export function useWebVaultDesktop(): WebVaultDesktop {
  return useWebVault() as WebVaultDesktop;
}
