import { APIRequestContext } from '@playwright/test';

export class DebugClient {
  request: APIRequestContext;
  baseUrl: string;

  constructor(request: APIRequestContext, baseUrl: string) {
    this.request = request;
    this.baseUrl = baseUrl;
  }

  async reset(): Promise<void> {
    await this.request.get(`${this.baseUrl}/debug/reset`);
  }

  async queueEnable(): Promise<void> {
    await this.request.get(`${this.baseUrl}/debug/queue/enable`);
  }

  async queueDisable(): Promise<void> {
    await this.request.get(`${this.baseUrl}/debug/queue/disable`);
  }

  async queueNext(status?: number): Promise<void> {
    await this.request.get(
      `${this.baseUrl}/debug/queue/next` +
        (status !== undefined ? `?status=${status}` : ''),
    );
  }

  async state(): Promise<{
    queueEnabled: boolean;
    queueRequests: [{ method: string; url: string }];
    pauseEnabled: boolean;
    downloadsPauseEnabled: boolean;
    uploadsPauseEnabled: boolean;
  }> {
    const res = await this.request.get(`${this.baseUrl}/debug/state.json`);
    return await res.json();
  }

  async withQueue(
    callback: (request: { method: string; url: string }) => Promise<boolean>,
    meanwhile?: () => Promise<void>,
  ) {
    await this.queueEnable();

    const run = async () => {
      // eslint-disable-next-line no-constant-condition
      while (true) {
        const state = await this.state();

        for (const request of state.queueRequests) {
          if (!(await callback(request))) {
            await this.queueDisable();

            return;
          }
        }

        await new Promise((resolve) => setTimeout(resolve, 50));
      }
    };

    const runPromise = run();

    if (meanwhile !== undefined) {
      await meanwhile();
    }

    await runPromise;
  }

  async createTestVaultRepo(): Promise<string> {
    return await this.request
      .get(`${this.baseUrl}/debug/vault/repos/create`)
      .then((res) => res.text());
  }
}
