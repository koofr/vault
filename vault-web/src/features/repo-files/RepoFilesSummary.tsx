import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';
import { FormattedMessage } from 'react-intl';

import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';

export const RepoFilesSummary = memo<{ info: RepoFilesBrowserInfo }>(
  ({ info }) => {
    const theme = useTheme();
    const {
      status,
      totalCount,
      totalSizeDisplay,
      selectedCount,
      selectedSizeDisplay,
    } = info;

    if (
      (status.type === 'Loading' && !status.loaded) ||
      status.type === 'Error' ||
      totalCount === 0
    ) {
      return null;
    }

    return (
      <div
        className={css`
          font-size: 13px;
          font-weight: normal;
          color: ${theme.colors.textLight};
          flex-shrink: 0;
        `}
      >
        {selectedCount > 0 ? (
          <span>
            <FormattedMessage
              id="web.repo_files_summary.selected_count.text"
              description="Summary text in the file list header showing selected item count and total size."
              defaultMessage="{count, plural, one {# item} other {# items}} - {size} selected"
              values={{ count: selectedCount, size: selectedSizeDisplay }}
            />
          </span>
        ) : (
          <span>
            <FormattedMessage
              id="web.repo_files_summary.total_count.text"
              description="Summary text in the file list header showing total item count and size."
              defaultMessage="{count, plural, one {# item} other {# items}} - {size}"
              values={{ count: totalCount, size: totalSizeDisplay }}
            />
          </span>
        )}
      </div>
    );
  },
);
RepoFilesSummary.displayName = 'RepoFilesSummary';
