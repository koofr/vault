import { memo } from 'react';

import { LoadingCircle } from '../../components/LoadingCircle';
import { PdfViewer } from '../../components/PdfViewer';

import { useRepoFilesDetailsBlobUrl } from './useRepoFilesDetailsBlobUrl';

export const RepoFilesDetailsPdfViewer = memo<{
  detailsId: number;
  width: number;
  height: number;
}>(({ detailsId, width, height }) => {
  const blobUrl = useRepoFilesDetailsBlobUrl(detailsId);

  return blobUrl !== undefined ? (
    <PdfViewer url={blobUrl} width={width} height={height} />
  ) : (
    <LoadingCircle />
  );
});
