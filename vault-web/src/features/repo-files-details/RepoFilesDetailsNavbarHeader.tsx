import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

import { Since } from '../../components/Since';
import { RepoFilesDetailsInfo } from '../../vault-wasm/vault-wasm';

export const RepoFilesDetailsNavbarHeader = memo<{
  info: RepoFilesDetailsInfo;
}>(({ info }) => {
  const intl = useIntl();
  const theme = useTheme();
  const {
    status,
    fileName,
    fileModified,
    isEditing,
    contentStatus,
    saveStatus,
    isDirty,
    error,
  } = info;
  const isLoading =
    status.type === 'Loading' || contentStatus.type === 'Loading';
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
      {isLoading ? (
        <div
          className={css`
            font-size: 10px;
            font-weight: normal;
          `}
        >
          <FormattedMessage
            id="web.repo_files_details.navbar_header.loading.text"
            description="Status text in the file details navbar while content is loading."
            defaultMessage="Loading…"
          />
        </div>
      ) : isSaving ? (
        <div
          className={css`
            font-size: 10px;
            font-weight: normal;
          `}
        >
          <FormattedMessage
            id="web.repo_files_details.navbar_header.saving.text"
            description="Status text in the file details navbar while changes are saving."
            defaultMessage="Saving…"
          />
        </div>
      ) : error !== undefined ? (
        <div
          className={css`
            font-size: 10px;
            font-weight: 600;
            color: ${theme.colors.destructive};
          `}
          aria-label={intl.formatMessage({
            id: 'web.repo_files_details.navbar_header.error.aria_label',
            description:
              'Accessibility label for the error message shown in the file details navbar.',
            defaultMessage: 'File error',
          })}
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
          aria-label={intl.formatMessage({
            id: 'web.repo_files_details.navbar_header.status.aria_label',
            description:
              'Accessibility label for the file status area in the file details navbar.',
            defaultMessage: 'File status',
          })}
        >
          <span
            className={css`
              font-size: 10px;
              font-weight: normal;
            `}
          >
            {fileModified !== undefined ? (
              <FormattedMessage
                id="web.repo_files_details.navbar_header.auto_save.text"
                description="Auto-save status line showing last saved time in file details."
                defaultMessage="Changes are saved automatically. Last saved {modified}"
                values={{
                  modified: <Since value={fileModified} />,
                }}
              />
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
                `,
            )}
            aria-label={
              isDirty
                ? intl.formatMessage({
                    id: 'web.repo_files_details.navbar_header.status.dirty',
                    description:
                      'Accessibility label for the dirty status indicator in file details.',
                    defaultMessage: 'File modified',
                  })
                : intl.formatMessage({
                    id: 'web.repo_files_details.navbar_header.status.unchanged',
                    description:
                      'Accessibility label for the unchanged status indicator in file details.',
                    defaultMessage: 'File unchanged',
                  })
            }
          ></span>
        </div>
      ) : null}
    </div>
  );
});
RepoFilesDetailsNavbarHeader.displayName = 'RepoFilesDetailsNavbarHeader';
