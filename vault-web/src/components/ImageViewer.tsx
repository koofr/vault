import { css, cx } from '@emotion/css';
import { memo } from 'react';

export interface ImageViewerProps {
  fileName: string;
  blobUrl: string;
  width: number;
  height: number;
}

export const ImageViewer = memo<ImageViewerProps>(
  ({ fileName, blobUrl, width, height }) => {
    return (
      <div
        className={cx(
          css`
            display: flex;
            flex-direction: row;
            justify-content: center;
            align-items: center;
            padding: 25px;
          `
        )}
        style={{
          width: `${width}px`,
          height: `${height}px`,
        }}
      >
        <img
          src={blobUrl}
          alt={fileName}
          className={css`
            width: 100%;
            height: 100%;
            object-fit: scale-down;
          `}
        />
      </div>
    );
  }
);
