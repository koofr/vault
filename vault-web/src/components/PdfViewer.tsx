import { css } from '@emotion/css';
import { memo } from 'react';

export const pdfViewerUrl = (fileUrl: string): string =>
  '/pdfjs-4.2.67/web/viewer.html?file=' + encodeURIComponent(fileUrl);

export const PdfViewer = memo<{
  url: string;
  width: number;
  height: number;
}>(({ url, width, height }) => {
  const viewerUrl = pdfViewerUrl(url);

  return (
    <iframe
      title="PDF viewer"
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
