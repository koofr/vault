import { css } from '@emotion/css';
import { memo } from 'react';
import { useIntl } from 'react-intl';

export const pdfViewerUrl = (fileUrl: string): string =>
  '/pdfjs-4.2.67/web/viewer.html?file=' + encodeURIComponent(fileUrl);

export const PdfViewer = memo<{
  url: string;
  width: number;
  height: number;
}>(({ url, width, height }) => {
  const intl = useIntl();
  const viewerUrl = pdfViewerUrl(url);

  return (
    <iframe
      title={intl.formatMessage({
        id: 'web.pdf_viewer.tooltip',
        description: 'Iframe title for the embedded PDF viewer.',
        defaultMessage: 'PDF viewer',
      })}
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
PdfViewer.displayName = 'PdfViewer';
