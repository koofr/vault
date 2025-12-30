import { WebVaultClient } from './WebVaultClient';
import { WebVaultDesktop } from './WebVaultDesktop';

const asyncCalls = new Set([
  'oauth2FinishFlowUrl',
  'repoRemoveRemove',
  'repoFilesBrowsersCreateFile',
]);

const bytesCalls = new Set([
  'userGetProfilePicture',
  'repoFilesDetailsContentBytesData',
]);

export function createProxy(client: WebVaultClient): WebVaultDesktop {
  return new Proxy(
    {
      client,
    },
    {
      get: function (target, prop) {
        if (Object.prototype.hasOwnProperty.call(target, prop)) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
          return (target as any)[prop];
        }

        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        return function (...args: any[]) {
          const name = prop as string;
          const asyncCall = asyncCalls.has(name);
          const bytes = bytesCalls.has(name);
          // eslint-disable-next-line @typescript-eslint/no-unsafe-return
          return client.call(name, Array.from(args), asyncCall, bytes);
        };
      },
    },
  ) as WebVaultDesktop;
}
