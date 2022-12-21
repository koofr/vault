import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import emptyFolderImage from '../../assets/images/empty-folder@2x.png';
import errorIconImage from '../../assets/images/error-icon@2x.png';
import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';

import { useRepoFilesUploadApi } from './RepoFilesUploadForm';

export const RepoFilesError = memo<{ info: RepoFilesBrowserInfo }>(
  ({ info }) => {
    const theme = useTheme();
    const uploadApi = useRepoFilesUploadApi();

    return (
      <div
        className={css`
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 80px 0 0;
        `}
      >
        {info.status.type === 'Loaded' && info.totalCount === 0 ? (
          <>
            <img
              src={emptyFolderImage}
              alt=""
              className={css`
                display: block;
                width: 252px;
                height: 186px;
                cursor: pointer;
              `}
              onClick={() => {
                uploadApi.uploadFile?.();
              }}
            />
            <h3
              className={css`
                font-size: 14px;
                color: ${theme.colors.text};
                font-weight: 600;
                margin: 0 0 20px;
              `}
            >
              This folder is empty.
            </h3>
          </>
        ) : info.status.type === 'Error' ? (
          <>
            <img
              src={errorIconImage}
              alt=""
              className={css`
                display: block;
                width: 290px;
                height: 194px;
              `}
            />
            <h3
              className={css`
                font-size: 14px;
                color: ${theme.colors.text};
                font-weight: 600;
                margin: 0 0 20px;
              `}
            >
              {info.status.error}
            </h3>
          </>
        ) : null}
      </div>
    );
  }
);
