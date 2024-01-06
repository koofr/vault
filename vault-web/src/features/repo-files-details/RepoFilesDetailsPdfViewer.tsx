import { memo } from 'react';

import { LoadingCircle } from '../../components/LoadingCircle';
import { PdfViewer } from '../../components/PdfViewer';

import { useRepoFilesDetailsFileUrl } from './useRepoFilesDetailsFileUrl';

export const RepoFilesDetailsPdfViewer = memo<{
  detailsId: number;
  width: number;
  height: number;
}>(({ detailsId, width, height }) => {
  const url = useRepoFilesDetailsFileUrl(detailsId);

  return url !== undefined ? (
    <PdfViewer url={url} width={width} height={height} />
  ) : (
    <LoadingCircle />
  );
});
