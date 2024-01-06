import { memo } from 'react';

import { ImageViewer } from '../../components/ImageViewer';
import { LoadingCircle } from '../../components/LoadingCircle';
import { Status } from '../../vault-wasm/vault-wasm';

import { useRepoFilesDetailsFileUrl } from './useRepoFilesDetailsFileUrl';

export const RepoFilesDetailsImageViewer = memo<{
  detailsId: number;
  fileName: string;
  contentStatus: Status | undefined;
  width: number;
  height: number;
}>(({ detailsId, fileName, contentStatus, width, height }) => {
  const url = useRepoFilesDetailsFileUrl(detailsId);

  return contentStatus === undefined ||
    (contentStatus.type === 'Loading' && !contentStatus.loaded) ||
    url === undefined ? (
    <LoadingCircle />
  ) : (
    <ImageViewer fileName={fileName} url={url} width={width} height={height} />
  );
});
