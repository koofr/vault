import { createContext } from 'react';

import { WebVault } from '../vault-wasm/vault-wasm';

// eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
export const WebVaultContext = createContext<WebVault>(undefined as any);
