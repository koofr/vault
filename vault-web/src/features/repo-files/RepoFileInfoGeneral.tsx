import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { format } from 'date-fns/format';
import { PropsWithChildren, memo } from 'react';
import { useIntl, IntlShape } from 'react-intl';

import { Since } from '../../components/Since';
import { FileCategory, RepoFile } from '../../vault-wasm/vault-wasm';

import { useDateFnsLocale } from '../intl/DateFnsLocaleContext';

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
Item.displayName = 'Item';

export const RepoFileInfoGeneral = memo<{ file: RepoFile }>(({ file }) => {
  const theme = useTheme();
  const intl = useIntl();
  const dateFnsLocale = useDateFnsLocale();

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
      `}
    >
      <Item
        label={intl.formatMessage({
          id: 'web.repo_file_info.name.label',
          description: 'Field label for the file name in the file info panel.',
          defaultMessage: 'Name',
        })}
      >
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
      <Item
        label={intl.formatMessage({
          id: 'web.repo_file_info.type.label',
          description:
            'Field label for the file type/category in the file info panel.',
          defaultMessage: 'Type',
        })}
      >
        {getCategoryDisplay(file.category, intl)}
      </Item>
      {file.sizeDisplay !== '' ? (
        <Item
          label={intl.formatMessage({
            id: 'web.repo_file_info.size.label',
            description:
              'Field label for the file size in the file info panel.',
            defaultMessage: 'Size',
          })}
        >
          {file.sizeDisplay}
        </Item>
      ) : null}
      {file.modified !== undefined ? (
        <Item
          label={intl.formatMessage({
            id: 'web.repo_file_info.modified.label',
            description:
              'Field label for the last modified date in the file info panel.',
            defaultMessage: 'Modified',
          })}
        >
          <Since value={file.modified} noTooltip />
          <br />
          {format(file.modified, 'PPPPpp', { locale: dateFnsLocale })}
        </Item>
      ) : null}
      <Item
        label={intl.formatMessage({
          id: 'web.repo_file_info.path.label',
          description:
            'Field label for the decrypted file path in the file info panel.',
          defaultMessage: 'Path',
        })}
      >
        {file.decryptedPath ?? '???'}
      </Item>
      <Item
        label={intl.formatMessage({
          id: 'web.repo_file_info.encrypted_path.label',
          description:
            'Field label for the encrypted file path in the file info panel.',
          defaultMessage: 'Encrypted path',
        })}
      >
        {file.encryptedPath}
      </Item>
      {file.type === 'File' ? (
        <>
          <Item
            label={intl.formatMessage({
              id: 'web.repo_file_info.md5.label',
              description:
                'Field label for the file MD5 hash in the file info panel.',
              defaultMessage: 'MD5',
            })}
          >
            {file.tags.hash ?? '???'}
          </Item>
          <Item
            label={intl.formatMessage({
              id: 'web.repo_file_info.encrypted_md5.label',
              description:
                'Field label for the encrypted MD5 hash in the file info panel.',
              defaultMessage: 'Encrypted MD5',
            })}
          >
            {file.remoteHash ?? '???'}
          </Item>
        </>
      ) : null}
      {file.tags.error !== undefined ? (
        <Item
          label={intl.formatMessage({
            id: 'web.repo_file_info.tags_error.label',
            description:
              'Field label for tag processing errors in the file info panel.',
            defaultMessage: 'Tags error',
          })}
        >
          <span
            className={css`
              color: ${theme.colors.destructive};
            `}
          >
            {file.tags.error}
          </span>
        </Item>
      ) : null}
    </div>
  );
});
RepoFileInfoGeneral.displayName = 'RepoFileInfoGeneral';

export function getCategoryDisplay(
  category: FileCategory,
  intl: IntlShape,
): string {
  switch (category) {
    case 'Generic':
      return intl.formatMessage({
        id: 'web.repo_file_category.file',
        description:
          'File category label for generic files in the file info panel.',
        defaultMessage: 'File',
      });
    case 'Folder':
      return intl.formatMessage({
        id: 'web.repo_file_category.folder',
        description: 'File category label for folders in the file info panel.',
        defaultMessage: 'Folder',
      });
    case 'Archive':
      return intl.formatMessage({
        id: 'web.repo_file_category.archive',
        description:
          'File category label for archive files in the file info panel.',
        defaultMessage: 'Archive',
      });
    case 'Audio':
      return intl.formatMessage({
        id: 'web.repo_file_category.audio',
        description:
          'File category label for audio files in the file info panel.',
        defaultMessage: 'Audio',
      });
    case 'Code':
      return intl.formatMessage({
        id: 'web.repo_file_category.code',
        description:
          'File category label for code files in the file info panel.',
        defaultMessage: 'Code',
      });
    case 'Document':
      return intl.formatMessage({
        id: 'web.repo_file_category.document',
        description:
          'File category label for document files in the file info panel.',
        defaultMessage: 'Document',
      });
    case 'Image':
      return intl.formatMessage({
        id: 'web.repo_file_category.image',
        description:
          'File category label for image files in the file info panel.',
        defaultMessage: 'Image',
      });
    case 'Pdf':
      return intl.formatMessage({
        id: 'web.repo_file_category.pdf',
        description:
          'File category label for PDF files in the file info panel.',
        defaultMessage: 'PDF',
      });
    case 'Presentation':
      return intl.formatMessage({
        id: 'web.repo_file_category.presentation',
        description:
          'File category label for presentation files in the file info panel.',
        defaultMessage: 'Presentation',
      });
    case 'Sheet':
      return intl.formatMessage({
        id: 'web.repo_file_category.spreadsheet',
        description:
          'File category label for spreadsheet files in the file info panel.',
        defaultMessage: 'Spreadsheet',
      });
    case 'Text':
      return intl.formatMessage({
        id: 'web.repo_file_category.text',
        description:
          'File category label for plain text files in the file info panel.',
        defaultMessage: 'Text',
      });
    case 'Video':
      return intl.formatMessage({
        id: 'web.repo_file_category.video',
        description:
          'File category label for video files in the file info panel.',
        defaultMessage: 'Video',
      });
  }
}
