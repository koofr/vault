import { createContext, useContext } from 'react';

export interface Config {
  baseUrl: string;
  oauth2ClientId: string;
  oauth2ClientSecret: string;
  appStoreUrl?: string;
  googlePlayUrl?: string;
}

export async function loadConfig(): Promise<Config> {
  const res = await fetch('/config.json');
  const resJson = await res.json();
  return resJson as Config;
}

export const ConfigContext = createContext<Config>(undefined as any);

export function useConfig() {
  return useContext(ConfigContext);
}
