import { createContext } from 'react';

import { WebVault } from '../vault-wasm/vault-wasm';

export const WebVaultContext = createContext<WebVault>(undefined as any);
