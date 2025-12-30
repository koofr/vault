import { createContext, useContext } from 'react';

export interface Config {
  baseUrl: string;
  oauth2ClientId: string;
  oauth2ClientSecret: string;
  appStoreUrl?: string;
  googlePlayUrl?: string;
  fDroidUrl?: string;
}

export async function loadConfig(): Promise<Config> {
  const res = await fetch('/config.json');
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const resJson: Config = await res.json();
  return resJson;
}

// eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
export const ConfigContext = createContext<Config>(undefined as any);

export function useConfig() {
  return useContext(ConfigContext);
}
