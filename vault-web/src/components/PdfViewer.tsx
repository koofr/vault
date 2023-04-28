import { css } from '@emotion/css';
import { memo } from 'react';

export const pdfViewerUrl = (fileUrl: string): string =>
  '/pdfjs-3.5.141/web/viewer.html?file=' + encodeURIComponent(fileUrl);

export const PdfViewer = memo<{
  url: string;
  width: number;
  height: number;
}>(({ url, width, height }) => {
  const viewerUrl = pdfViewerUrl(url);

  return (
    <iframe
      title="Viewer"
      id="viewerIframe"
      src={viewerUrl}
      width={width}
      height={height}
      className={css`
        border: none;
        display: block;
      `}
    />
  );
});
