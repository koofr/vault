import { test as base } from '@playwright/test';
import { readFileSync, writeFileSync } from 'fs';
import path from 'path';
import { Agent } from 'undici';

import { oauth2RedirectUri } from './baseUrl';
import { OAuth2Token } from './oauth2';
import { config, ignoreHTTPSErrors } from './vaultConfig';

export const workersCount: number = process.env.CI ? 1 : 1;
export const workers: number[] = Array.from(Array(workersCount), (_, i) => i);
export const storageStatePaths: string[] =
  workersCount === 1
    ? [path.join(__dirname, '../playwright/.auth/user.json')]
    : workers.map((idx) =>
        // user-0.json to user-7.json
        path.join(__dirname, `../playwright/.auth/user-${idx}.json`)
      );

export function getCurrentWorker(): number {
  return base.info().parallelIndex;
}

export function getStorageStatePath(worker: number): string {
  return storageStatePaths[worker];
}

export function getCurrentWorkerStorageStatePath(): string {
  return getStorageStatePath(getCurrentWorker());
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getStorageState(worker: number): any {
  try {
    return JSON.parse(
      readFileSync(getStorageStatePath(worker)).toString('utf8')
    );
  } catch {
    throw new Error(`Missing file ${getStorageStatePath(worker)}`);
  }
}

export function getOAuth2Token(worker: number): OAuth2Token {
  const storageState = getStorageState(worker);

  return JSON.parse(storageState.origins[0].localStorage[0].value);
}

export function getCurrentWorkerOAuth2Token(): OAuth2Token {
  return getOAuth2Token(getCurrentWorker());
}

export function setOAuth2Token(worker: number, oauth2Token: OAuth2Token) {
  const storageState = getStorageState(worker);

  storageState.origins[0].localStorage[0].value = JSON.stringify(oauth2Token);

  writeFileSync(
    getStorageStatePath(worker),
    JSON.stringify(storageState, null, 2)
  );
}

export async function refreshOAuth2Tokens(force: boolean): Promise<void> {
  for (const worker of workers) {
    await refreshOAuth2Token(worker, force);
  }
}

async function refreshOAuth2Token(
  worker: number,
  force: boolean
): Promise<void> {
  const oauth2Token = getOAuth2Token(worker);

  if (!force) {
    // skip if the current access token is valid for more than 10 minutes
    if (oauth2Token.expires_at > Date.now() + 10 * 60 * 1000) {
      return;
    }
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

  const refreshedToken: OAuth2Token = {
    access_token: rawToken.access_token,
    refresh_token: rawToken.refresh_token,
    expires_at: Date.now() + rawToken.expires_in * 1000,
  };

  setOAuth2Token(worker, refreshedToken);
}
