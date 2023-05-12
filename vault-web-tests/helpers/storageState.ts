import { readFileSync, writeFileSync } from 'fs';
import path from 'path';
import { Agent } from 'undici';

import { oauth2RedirectUri } from './baseUrl';
import { OAuth2Token } from './oauth2';
import { config, ignoreHTTPSErrors } from './vaultConfig';

export const storageStatePath = path.join(
  __dirname,
  '../playwright/.auth/user.json'
);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getStorageState(): any {
  try {
    return JSON.parse(readFileSync(storageStatePath).toString('utf8'));
  } catch {
    throw new Error(`Missing file ${storageStatePath}`);
  }
}

export function getOAuth2Token(): OAuth2Token {
  const storageState = getStorageState();

  return JSON.parse(storageState.origins[0].localStorage[0].value);
}

export function setOAuth2Token(oauth2Token: OAuth2Token) {
  const storageState = getStorageState();

  storageState.origins[0].localStorage[0].value = JSON.stringify(oauth2Token);

  writeFileSync(storageStatePath, JSON.stringify(storageState, null, 2));
}

export async function refreshOAuth2Token() {
  const oauth2Token = getOAuth2Token();

  // skip if the current access token is valid for more than 10 minutes
  if (oauth2Token.expires_at > Date.now() + 10 * 60 * 1000) {
    return;
  }

  const request: RequestInit = {
    method: 'POST',
    body: new URLSearchParams({
      grant_type: 'refresh_token',
      client_id: config.oauth2ClientId,
      client_secret: config.oauth2ClientSecret,
      redirect_uri: oauth2RedirectUri,
      refresh_token: oauth2Token.refresh_token,
    }),
  };

  if (ignoreHTTPSErrors) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (request as any).dispatcher = new Agent({
      connect: {
        rejectUnauthorized: false,
      },
    });
  }

  const res = await fetch(`${config.baseUrl}/oauth2/token`, request);

  if (res.status != 200) {
    throw new Error(
      `Failed to refresh oauth2 token: ${res.status} ${await res.text()}`
    );
  }

  const rawToken: {
    access_token: string;
    refresh_token: string;
    expires_in: number;
  } = await res.json();

  setOAuth2Token({
    access_token: rawToken.access_token,
    refresh_token: rawToken.refresh_token,
    expires_at: Date.now() + rawToken.expires_in * 1000,
  });
}
