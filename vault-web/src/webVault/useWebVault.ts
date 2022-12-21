import { useContext } from 'react';

import { WebVault } from '../vault-wasm/vault-wasm';

import { WebVaultContext } from './webVaultContext';

export function useWebVault(): WebVault {
  const webVault = useContext(WebVaultContext);

  return webVault;
}
