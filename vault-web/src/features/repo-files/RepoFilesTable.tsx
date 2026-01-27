import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, MouseEvent, useCallback, useMemo } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';
import { Link } from 'react-router-dom';

import FilesRenameHoverIcon from '../../assets/images/files-rename-hover.svg?react';
import FilesRenameIcon from '../../assets/images/files-rename.svg?react';
import { Since } from '../../components/Since';
import { FileIcon } from '../../components/file-icon/FileIcon';
import {
  Column,
  RowProps,
  Table,
  TableRow,
} from '../../components/table/Table';
import { useIsMobile } from '../../components/useIsMobile';
import { buttonReset } from '../../styles/mixins/buttons';
import { allStates } from '../../styles/mixins/hover';
import { isExtend, isRange } from '../../utils/selectionEvents';
import {
  RepoFile,
  RepoFilesBrowserInfo,
  RepoFilesBrowserItem,
} from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { openFile } from './repoFilesActions';
import {
  fileHasDetails,
  repoFilesDetailsLink,
  repoFilesLink,
} from './selectors';

interface TableData {
  items: RepoFilesBrowserItem[];
}

const FileName = memo<{ file: RepoFile }>(({ file }) => {
  const intl = useIntl();
  const isMobile = useIsMobile();
  const theme = useTheme();
  const webVault = useWebVault();
  const onClick = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    openFile(webVault, file.repoId, file.encryptedPath, isMobile);
  }, [webVault, file, isMobile]);
  const renameFile = useCallback(() => {
    webVault.repoFilesRenameFile(file.repoId, file.encryptedPath);
  }, [webVault, file]);

  const name =
    file.type === 'Dir' ? (
      <Link
        to={repoFilesLink(file.repoId, file.encryptedPath)}
        className={css`
          font-weight: 600;

          ${allStates} {
            color: ${theme.colors.text};
          }
        `}
      >
        {file.name}
      </Link>
    ) : fileHasDetails(file) ? (
      <Link
        to={repoFilesDetailsLink(file.repoId, file.encryptedPath)}
        className={css`
          ${allStates} {
            color: ${theme.colors.text};
          }
        `}
      >
        {file.name}
      </Link>
    ) : (
      <a
        href="."
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          onClick();
        }}
        className={css`
          ${allStates} {
            color: ${theme.colors.text};
          }
        `}
      >
        {file.name}
      </a>
    );

  return (
    <div
      className={css`
        display: flex;
        flex-direction: row;
        align-items: center;
      `}
    >
      <span
        className={css`
          margin-right: 15px;
        `}
      >
        <FileIcon size="Sm" attrs={file.fileIconAttrs} />
      </span>
      <span
        className={cx(
          css`
            white-space: nowrap;
            text-overflow: ellipsis;
            overflow: hidden;
          `,
          file.nameError !== undefined &&
            css`
              color: ${theme.colors.destructive};
            `,
        )}
      >
        {file.nameError !== undefined ? (
          <FormattedMessage
            id="web.repo_files.name_with_error.text"
            description="File name display when the name has an error, adding an ERROR suffix."
            defaultMessage="{name} (ERROR)"
            values={{ name }}
          />
        ) : (
          name
        )}
      </span>
      {!isMobile ? (
        <button
          className={css`
            ${buttonReset}
            width: 32px;
            height: 32px;
            display: flex;
            justify-content: center;
            align-items: center;
            position: relative;
            top: 2px;
            display: none;

            *:hover > * > * > * > & {
              display: block;
            }
          `}
          onClick={(e) => {
            e.stopPropagation();

            renameFile();
          }}
          title={intl.formatMessage({
            id: 'web.repo_files.rename.tooltip',
            description:
              'Tooltip for the rename icon button shown on file rows.',
            defaultMessage: 'Rename',
          })}
        >
          <FilesRenameIcon
            className={css`
              button:hover > & {
                display: none;
              }
            `}
            role="img"
          />
          <FilesRenameHoverIcon
            className={css`
              display: none;

              button:hover > & {
                display: inline;
              }
            `}
            role="img"
          />
        </button>
      ) : null}
    </div>
  );
});
FileName.displayName = 'FileName';

export const FileSize = memo<{ file: RepoFile }>(({ file }) => {
  return (
    <span
      className={css`
        font-size: 12px;
      `}
    >
      {file.sizeDisplay}
    </span>
  );
});
FileSize.displayName = 'FileSize';

export const FileModified = memo<{ file: RepoFile }>(({ file }) => {
  if (file.modified === undefined) {
    return null;
  }

  return (
    <span
      className={css`
        font-size: 12px;
      `}
    >
      <Since value={file.modified} />
    </span>
  );
});
FileModified.displayName = 'FileModified';

const RepoFilesTableRow = memo<RowProps<TableData>>(({ index, data }) => {
  const intl = useIntl();
  const item = useMemo(() => data.items[index], [data, index]);
  const [file] = useSubscribe(
    (v, cb) => v.repoFilesFileSubscribe(item.fileId, cb),
    (v) => v.repoFilesFileData,
    [item.fileId],
  );
  const isSelected = item.isSelected;
  const isFirstSelected =
    isSelected && (index === 0 || !data.items[index - 1].isSelected);
  const row = useMemo(() => {
    if (file === undefined) {
      return undefined;
    }

    return {
      name: <FileName file={file} />,
      size: <FileSize file={file} />,
      modified: <FileModified file={file} />,
    };
  }, [file]);

  if (row === undefined) {
    return null;
  }

  return (
    <TableRow
      key={item.fileId}
      index={index}
      row={row}
      isSelected={isSelected}
      isFirstSelected={isFirstSelected}
      ariaLabel={
        file !== undefined
          ? file.type === 'Dir'
            ? intl.formatMessage(
                {
                  id: 'web.repo_files.dir.aria_label',
                  description:
                    'Accessibility label for a folder row in the files table.',
                  defaultMessage: 'Folder {name}',
                },
                { name: file.name },
              )
            : intl.formatMessage(
                {
                  id: 'web.repo_files.file.aria_label',
                  description:
                    'Accessibility label for a file row in the files table.',
                  defaultMessage: 'File {name}',
                },
                { name: file.name },
              )
          : undefined
      }
    />
  );
});
RepoFilesTableRow.displayName = 'RepoFilesTableRow';

export const RepoFilesTable = memo<{
  info: RepoFilesBrowserInfo;
}>(({ info }) => {
  const intl = useIntl();
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();
  const items = info.items;
  const data = useMemo(
    (): TableData => ({
      items,
    }),
    [items],
  );
  const sort = info.sort;
  const columns = useMemo(
    (): Column[] => [
      {
        name: 'name',
        label: intl.formatMessage({
          id: 'web.repo_files.name.column',
          description:
            'Column header label for the file name in the files table.',
          defaultMessage: 'Name',
        }),
        sortBy: sort.field === 'Name' ? sort.direction : 'Hidden',
      },
      {
        name: 'size',
        label: intl.formatMessage({
          id: 'web.repo_files.size.column',
          description: 'Column header label for file size in the files table.',
          defaultMessage: 'Size',
        }),
        width: isMobile ? 0 : '15%',
        minWidth: isMobile ? undefined : 70,
        sortBy: sort.field === 'Size' ? sort.direction : 'Hidden',
      },
      {
        name: 'modified',
        label: intl.formatMessage({
          id: 'web.repo_files.modified.column',
          description:
            'Column header label for last modified date in the files table.',
          defaultMessage: 'Modified',
        }),
        width: isMobile ? 0 : '20%',
        minWidth: isMobile ? undefined : 150,
        sortBy: sort.field === 'Modified' ? sort.direction : 'Hidden',
      },
    ],
    [isMobile, sort, intl],
  );
  const onHeadCheckboxClick = useCallback(() => {
    if (info.selectionSummary === 'All') {
      webVault.repoFilesBrowsersClearSelection(browserId);
    } else {
      webVault.repoFilesBrowsersSelectAll(browserId);
    }
  }, [webVault, browserId, info]);
  const onSortByClick = useCallback(
    (_: MouseEvent, columnName: string) => {
      switch (columnName) {
        case 'name':
          webVault.repoFilesBrowsersSortBy(browserId, 'Name');
          break;
        case 'size':
          webVault.repoFilesBrowsersSortBy(browserId, 'Size');
          break;
        case 'modified':
          webVault.repoFilesBrowsersSortBy(browserId, 'Modified');
          break;
      }
    },
    [webVault, browserId],
  );
  const onRowCheckboxClick = useCallback(
    (event: MouseEvent<HTMLElement>, index: number) => {
      event.stopPropagation();
      webVault.repoFilesBrowsersSelectFile(
        browserId,
        items[index].fileId,
        true,
        isRange(event),
        false,
      );
    },
    [webVault, browserId, items],
  );
  const onRowClick = useCallback(
    (event: MouseEvent<HTMLElement>, index: number) => {
      webVault.repoFilesBrowsersSelectFile(
        browserId,
        items[index].fileId,
        isExtend(event),
        isRange(event),
        false,
      );
    },
    [webVault, browserId, items],
  );
  const onRowContextMenu = useCallback(() => {}, []);

  return (
    <Table
      columns={columns}
      selectionSummary={info.selectionSummary}
      length={items.length}
      data={data}
      Row={RepoFilesTableRow}
      ariaLabel={intl.formatMessage({
        id: 'web.repo_files.table.aria_label',
        description: 'Accessibility label for the files table/list.',
        defaultMessage: 'Files list',
      })}
      onHeadCheckboxClick={onHeadCheckboxClick}
      onSortByClick={onSortByClick}
      onRowCheckboxClick={onRowCheckboxClick}
      onRowClick={onRowClick}
      onRowContextMenu={onRowContextMenu}
    />
  );
});
RepoFilesTable.displayName = 'RepoFilesTable';
