import { css } from '@emotion/css';
import { memo } from 'react';

import { FileIcon } from '../../components/file-icon/FileIcon';
import { RepoFile } from '../../vault-wasm/vault-wasm';

export const RepoFileInfoImage = memo<{ file: RepoFile }>(({ file }) => {
  return (
    <div
      className={css`
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        overflow-x: hidden;
      `}
    >
      <div
        className={css`
          margin: 25px 25px 50px;
        `}
      >
        <FileIcon size="Lg" attrs={file.fileIconAttrs} />
      </div>
    </div>
  );
});
