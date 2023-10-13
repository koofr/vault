import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import format from 'date-fns/format';
import { PropsWithChildren, memo } from 'react';

import { Since } from '../../components/Since';
import { FileCategory, RepoFile } from '../../vault-wasm/vault-wasm';

export const Item = memo<PropsWithChildren<{ label: string }>>(
  ({ label, children }) => {
    const theme = useTheme();

    return (
      <div
        className={css`
          margin: 0 0 15px;
        `}
      >
        <div
          className={css`
            font-size: 13px;
            font-weight: normal;
            color: ${theme.colors.textLight};
          `}
        >
          {label}
        </div>
        <div
          className={css`
            font-size: 13px;
            font-weight: normal;
            color: ${theme.colors.text};
            line-height: 1.7;
            word-break: break-word;

            & p {
              margin: 0 0 1em;

              &:last-child {
                margin-bottom: 0;
              }
            }
          `}
        >
          {children}
        </div>
      </div>
    );
  },
);

export const RepoFileInfoGeneral = memo<{ file: RepoFile }>(({ file }) => {
  const theme = useTheme();

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
      `}
    >
      <Item label="Name">
        {file.nameError === undefined ? (
          <>{file.name}</>
        ) : (
          <div
            className={css`
              color: ${theme.colors.destructive};
            `}
          >
            <p>{file.name}</p>
            <p>{file.nameError}</p>
          </div>
        )}
      </Item>
      <Item label="Type">{getCategoryDisplay(file.category)}</Item>
      {file.sizeDisplay !== '' ? (
        <Item label="Size">{file.sizeDisplay}</Item>
      ) : null}
      {file.modified !== undefined ? (
        <Item label={'Modified'}>
          <Since value={file.modified} noTooltip />
          <br />
          {format(file.modified, 'PPPPpp')}
        </Item>
      ) : null}
    </div>
  );
});

export function getCategoryDisplay(category: FileCategory): string {
  switch (category) {
    case 'Generic':
      return 'File';
    case 'Folder':
      return 'Folder';
    case 'Archive':
      return 'Archive';
    case 'Audio':
      return 'Audio';
    case 'Code':
      return 'Code';
    case 'Document':
      return 'Document';
    case 'Image':
      return 'Image';
    case 'Pdf':
      return 'PDF';
    case 'Presentation':
      return 'Presentation';
    case 'Sheet':
      return 'Spreadsheet';
    case 'Text':
      return 'Text';
    case 'Video':
      return 'Video';
  }
}
