import { APIRequestContext } from '@playwright/test';

export interface OAuth2Token {
  access_token: string;
  refresh_token: string;
}

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

  async refreshToken() {
    const res = await this.request.fetch(`${this.baseUrl}/oauth2/token`, {
      ignoreHTTPSErrors: this.ignoreHTTPSErrors,
      method: 'POST',
      data: {
        grant_type: 'refresh_token',
        client_id: this.oauth2ClientId,
        client_secret: this.oauth2ClientSecret,
        redirect_uri: this.oauth2RedirectUri,
        refresh_token: this.oauth2Token.refresh_token,
      },
    });

    if (res.status() != 200) {
      throw new Error(
        `Failed to refresh oauth2 token: ${res.status()} ${await res.text()}`
      );
    }

    this.oauth2Token = await res.json();
  }

  async removeAllVaultRepos() {
    const repos = await this.getVaultRepos();

    for (const repo of repos) {
      await this.deleteFile(repo.mountId, repo.path);
    }
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
}
