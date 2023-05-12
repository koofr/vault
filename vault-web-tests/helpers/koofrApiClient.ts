import { APIRequestContext } from '@playwright/test';

import { OAuth2Token } from './oauth2';

export interface VaultRepo {
  id: string;
  name: string;
  mountId: string;
  path: string;
}

export class KoofrApiClient {
  request: APIRequestContext;
  baseUrl: string;
  oauth2Token: OAuth2Token;
  oauth2ClientId: string;
  oauth2ClientSecret: string;
  oauth2RedirectUri: string;
  ignoreHTTPSErrors: boolean;

  constructor(
    request: APIRequestContext,
    baseUrl: string,
    oauth2Token: OAuth2Token,
    oauth2ClientId: string,
    oauth2ClientSecret: string,
    oauth2RedirectUri: string,
    ignoreHTTPSErrors: boolean
  ) {
    this.request = request;
    this.baseUrl = baseUrl;
    this.oauth2Token = oauth2Token;
    this.oauth2ClientId = oauth2ClientId;
    this.oauth2ClientSecret = oauth2ClientSecret;
    this.oauth2RedirectUri = oauth2RedirectUri;
    this.ignoreHTTPSErrors = ignoreHTTPSErrors;
  }

  async removeAllVaultRepos() {
    const repos = await this.getVaultRepos();

    for (const repo of repos) {
      await this.deleteFile(repo.mountId, repo.path);
    }
  }

  async createTestVaultRepo(): Promise<VaultRepo> {
    try {
      await this.createDir('primary', '/', 'My safe box');
    } catch (e) {
      if (/AlreadyExists/.test(e.message)) {
        await this.deleteFile('primary', '/My safe box');
        await this.createDir('primary', '/', 'My safe box');
      } else {
        throw e;
      }
    }

    return await this.createVaultRepo('primary', '/My safe box');
  }

  getRequestOptions(): {
    ignoreHTTPSErrors: boolean;
    headers: { [key: string]: string };
  } {
    return {
      ignoreHTTPSErrors: this.ignoreHTTPSErrors,
      headers: {
        Authorization: `Bearer ${this.oauth2Token.access_token}`,
      },
    };
  }

  async getVaultRepos(): Promise<VaultRepo[]> {
    const res = await this.request.get(`${this.baseUrl}/api/v2.1/vault/repos`, {
      ...this.getRequestOptions(),
    });

    if (res.status() != 200) {
      throw new Error(
        `Failed to get vault repos: ${res.status()} ${await res.text()}`
      );
    }

    const repos = await res.json();

    return repos.repos;
  }

  async deleteFile(mountId: string, path: string) {
    const res = await this.request.delete(
      `${
        this.baseUrl
      }/api/v2.1/mounts/${mountId}/files/remove?path=${encodeURIComponent(
        path
      )}`,
      {
        ...this.getRequestOptions(),
      }
    );

    if (res.status() != 200) {
      throw new Error(
        `Failed to delete path: ${res.status()} ${await res.text()}`
      );
    }
  }

  async createDir(mountId: string, parentPath: string, name: string) {
    const res = await this.request.post(
      `${
        this.baseUrl
      }/api/v2.1/mounts/${mountId}/files/folder?path=${encodeURIComponent(
        parentPath
      )}`,
      {
        ...this.getRequestOptions(),
        data: {
          name,
        },
      }
    );

    if (res.status() != 200) {
      throw new Error(
        `Failed to create dir: ${res.status()} ${await res.text()}`
      );
    }
  }

  async createVaultRepo(mountId: string, path: string): Promise<VaultRepo> {
    const res = await this.request.post(
      `${this.baseUrl}/api/v2.1/vault/repos`,
      {
        ...this.getRequestOptions(),
        data: {
          mountId,
          path,
          passwordValidator: '4febe12d-33f5-4a8e-b1d1-dee8a1ec0af3',
          passwordValidatorEncrypted:
            'v2:UkNMT05FAADskNtA1B4NLVspahPySD83io24-Aqq1g-O9dNR2MeE2_Sp9aL4rrURY_JuwH-ffpbAx5snpR2mwFFfAue3a25qAduOj_Uspu1kNLmX',
          salt: 'salt',
        },
      }
    );

    if (res.status() != 201) {
      throw new Error(
        `Failed to create vault repo: ${res.status()} ${await res.text()}`
      );
    }

    return await res.json();
  }
}
