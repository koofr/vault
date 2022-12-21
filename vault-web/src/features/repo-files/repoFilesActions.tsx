import { downloadStream } from '../../utils/downloadStream';
import { RepoFile, WebVault } from '../../vault-wasm/vault-wasm';

export const downloadFile = async (
  webVault: WebVault,
  file: RepoFile,
  isMobile: boolean
) => {
  const forceBlob = isMobile;
  const stream = await webVault.repoFilesGetFileStream(file.id, forceBlob);

  if (stream === undefined) {
    return;
  }

  downloadStream(stream, file.name);
};
