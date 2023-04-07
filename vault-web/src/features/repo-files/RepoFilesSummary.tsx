import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

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
      status.type === 'Loading' ||
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
            {selectedCount} {selectedCount === 1 ? 'item' : 'items'} -{' '}
            {selectedSizeDisplay} selected
          </span>
        ) : (
          <span>
            {totalCount} {totalCount === 1 ? 'item' : 'items'} -{' '}
            {totalSizeDisplay}
          </span>
        )}
      </div>
    );
  }
);
