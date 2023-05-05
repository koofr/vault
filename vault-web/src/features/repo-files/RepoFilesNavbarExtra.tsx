import { css, cx } from '@emotion/css';
import { memo } from 'react';

import { useIsMobile } from '../../components/useIsMobile';
import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';

import { RepoFilesSummary } from './RepoFilesSummary';
import { RepoFilesToolbar } from './RepoFilesToolbar';

export const RepoFilesNavbarExtra = memo<{
  info?: RepoFilesBrowserInfo;
}>(({ info }) => {
  const isMobile = useIsMobile();

  return (
    <div
      className={cx(
        css`
          display: flex;
        `,
        isMobile
          ? css`
              flex-direction: column;
              align-items: center;
            `
          : css`
              width: 100%;
              flex-direction: row;
              align-items: center;
              overflow: hidden;
            `
      )}
      >
      {/* TODO searchbox */}
      {/* <div
        className={cx(
          css`
            margin: 0;
            padding: 0;
            width: 250px;
            flex-grow: 0;
            flex-shrink: 0;
          `,
          isMobile
            ? css`
                display: none;
              `
            : css``
        )}
      >
      </div> */}
      {info !== undefined ? (
        <div
          className={cx(
            css`
              flex-grow: 1;
              flex-shrink: 0;
              margin: 0;
              padding: 0;
              display: flex;
              align-items: center;
              position: relative;
            `,
            isMobile
              ? css``
              : css`
                  margin-right: 50px;
                `
          )}
        >
          <RepoFilesSummary info={info} />
          <RepoFilesToolbar info={info} />
        </div>
      ) : null}
    </div>
  );
});
