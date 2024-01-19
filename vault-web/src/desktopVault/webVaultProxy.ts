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
        if (target.hasOwnProperty(prop)) {
          return (target as any)[prop];
        }

        return function () {
          const name = prop as string;
          const asyncCall = asyncCalls.has(name);
          const bytes = bytesCalls.has(name);
          return client.call(name, Array.from(arguments), asyncCall, bytes);
        };
      },
    },
  ) as WebVaultDesktop;
}
