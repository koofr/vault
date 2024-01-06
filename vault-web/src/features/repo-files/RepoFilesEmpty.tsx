import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import emptyFolderImage from '../../assets/images/empty-folder@2x.png';

import { useRepoFilesUploadApi } from './RepoFilesUploadApi';

export const RepoFilesEmpty = memo(() => {
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
    </div>
  );
});
