import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import { Since } from '../../../components/Since';
import { RepoFilesDetailsInfo } from '../../../vault-wasm/vault-wasm';

export const RepoFilesDetailsNavbarHeader = memo<{
  info: RepoFilesDetailsInfo;
}>(({ info }) => {
  const theme = useTheme();
  const { fileName, fileModified, isEditing, saveStatus, isDirty, error } =
    info;
  const isSaving = saveStatus.type === 'Loading';

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
      `}
    >
      <div
        className={css`
          font-weight: 600;
        `}
        aria-label="File name"
      >
        {fileName}
      </div>
      {isSaving ? (
        <div
          className={css`
            font-size: 10px;
            font-weight: normal;
          `}
        >
          Saving...
        </div>
      ) : error !== undefined ? (
        <div
          className={css`
            font-size: 10px;
            font-weight: 600;
            color: ${theme.colors.destructive};
          `}
          aria-label="File error"
        >
          {error}
        </div>
      ) : isEditing ? (
        <div
          className={css`
            display: flex;
            flex-direction: row;
            align-items: center;
          `}
          aria-label="File info"
        >
          <span
            className={css`
              font-size: 10px;
              font-weight: normal;
            `}
          >
            Changes are saved automatically.{' '}
            {fileModified !== undefined ? (
              <>
                Last saved <Since value={fileModified} />
              </>
            ) : null}
          </span>
          <span
            className={cx(
              css`
                margin-left: 10px;
                width: 8px;
                height: 8px;
                border-radius: 4px;
                background-color: ${theme.colors.successful};
              `,
              isDirty &&
                css`
                  background-color: ${theme.colors.warning};
                `
            )}
            aria-label={isDirty ? 'File modified' : 'File unchanged'}
          ></span>
        </div>
      ) : null}
    </div>
  );
});
