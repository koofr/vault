import { memo } from 'react';

import { ImageViewer } from '../../../components/ImageViewer';
import { LoadingCircle } from '../../../components/LoadingCircle';
import { Status } from '../../../vault-wasm/vault-wasm';

import { useRepoFilesDetailsBlobUrl } from './useRepoFilesDetailsBlobUrl';

export const RepoFilesDetailsImageViewer = memo<{
  detailsId: number;
  fileName: string;
  contentStatus: Status | undefined;
  width: number;
  height: number;
}>(({ detailsId, fileName, contentStatus, width, height }) => {
  const blobUrl = useRepoFilesDetailsBlobUrl(detailsId);

  return contentStatus === undefined ||
    contentStatus.type === 'Loading' ||
    blobUrl === undefined ? (
    <LoadingCircle />
  ) : (
    <ImageViewer
      fileName={fileName}
      blobUrl={blobUrl}
      width={width}
      height={height}
    />
  );
});
