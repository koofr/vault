import { getInitialOAuth2Token } from './oauth2';
import { config, ignoreHTTPSErrors } from './vaultConfig';
import { WebVaultClient } from './webVaultClient';

async function main() {
  const baseUrl = config.baseUrl;
  const oauth2Token = getInitialOAuth2Token();
  const oauth2ClientId = config.oauth2ClientId;
  const oauth2ClientSecret = config.oauth2ClientSecret;
  const oauth2RedirectUri = 'http://localhost:5173/oauth2callback';

  const client = new WebVaultClient(
    baseUrl,
    oauth2Token,
    oauth2ClientId,
    oauth2ClientSecret,
    oauth2RedirectUri,
    ignoreHTTPSErrors
  );

  await client.webVault.load();

  console.log('Loaded');

  const user = await client.waitFor(
    (v, cb) => v.userSubscribe(cb),
    (v) => v.userData,
    (user) => user !== undefined
  );

  console.log('User', user);

  const repos = await client.waitFor(
    (v, cb) => v.reposSubscribe(cb),
    (v) => v.reposData,
    (repos) => repos.status.type === 'Loaded'
  );

  console.log('Repos', repos);

  if (repos.repos.length === 0) {
    console.log('No repos');

    return;
  }

  const repo = repos.repos[0];

  const unlockId = client.webVault.repoUnlockCreate(repo.id, {
    mode: 'Unlock',
  });
  await client.webVault.repoUnlockUnlock(unlockId, 'password');
  client.webVault.repoUnlockDestroy(unlockId);

  const browserId = client.webVault.repoFilesBrowsersCreate(repo.id, '/', {
    selectName: undefined,
  });

  await client.waitFor(
    (v, cb) => v.repoFilesBrowsersInfoSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersInfoData,
    (info) => {
      return info.status.type === 'Loaded';
    }
  );

  let items = await client.waitFor(
    (v, cb) => v.repoFilesBrowsersItemsSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersItemsData,
    () => true
  );

  console.log('Files', items);

  const transfersUnsubscribe = client.subscribe(
    (v, cb) => v.transfersSummarySubscribe(cb),
    (v) => v.transfersSummaryData,
    (data) => {
      console.log('Transfers summary', data);
    }
  );

  const fileName = `${(Math.random() * 1000000000) >> 0}.txt`;
  const fileContent = new Blob(['test file']);

  console.log('Upload file');

  await client.webVault.transfersUpload(repo.id, '/', fileName, fileContent);

  console.log('Upload done');

  transfersUnsubscribe();

  items = await client.waitFor(
    (v, cb) => v.repoFilesBrowsersItemsSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersItemsData,
    (items) =>
      items.find((item) => item.fileId.endsWith(`/${fileName}`)) !== undefined
  );

  const fileId = items.find((item) =>
    item.fileId.endsWith(`/${fileName}`)
  ).fileId;
  const file = await client.waitFor(
    (v, cb) => v.repoFilesFileSubscribe(fileId, cb),
    (v) => v.repoFilesFileData,
    () => true
  );

  console.log('Uploaded file', file);

  console.log('Download file');

  const downloadStream = await client.webVault.repoFilesGetFileStream(
    file.repoId,
    file.path,
    false
  );

  console.log('Download done');

  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const downloadStreamText = await new Response(downloadStream.stream!).text();

  console.log('Downloaded content', downloadStream.name, downloadStreamText);
}

main().catch((e) => console.warn(e));
