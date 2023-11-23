import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, MouseEvent, useCallback, useMemo } from 'react';
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
  RepoFilesSortFieldArg,
} from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { downloadFile } from './repoFilesActions';
import {
  fileHasDetails,
  repoFilesDetailsLink,
  repoFilesLink,
} from './selectors';

interface TableData {
  items: RepoFilesBrowserItem[];
}

const FileName = memo<{ file: RepoFile }>(({ file }) => {
  const isMobile = useIsMobile();
  const theme = useTheme();
  const webVault = useWebVault();
  const onClick = useCallback(() => {
    downloadFile(webVault, file.repoId, file.encryptedPath, isMobile);
  }, [webVault, file, isMobile]);
  const renameFile = useCallback(() => {
    webVault.repoFilesRenameFile(file.repoId, file.encryptedPath);
  }, [webVault, file]);

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
        {file.type === 'Dir' ? (
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
        )}
        {file.nameError !== undefined ? ' (ERROR)' : null}
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
          aria-label="Rename"
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

const RepoFilesTableRow = memo<RowProps<TableData>>(({ index, data }) => {
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
            ? `Folder ${file.name}`
            : `File ${file.name}`
          : undefined
      }
    />
  );
});

export const RepoFilesTable = memo<{
  info: RepoFilesBrowserInfo;
}>(({ info }) => {
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
        label: 'Name',
        sortBy: sort.field === 'Name' ? sort.direction : 'Hidden',
      },
      {
        name: 'size',
        label: 'Size',
        width: isMobile ? 0 : '15%',
        minWidth: isMobile ? undefined : 70,
        sortBy: sort.field === 'Size' ? sort.direction : 'Hidden',
      },
      {
        name: 'modified',
        label: 'Modified',
        width: isMobile ? 0 : '20%',
        minWidth: isMobile ? undefined : 150,
        sortBy: sort.field === 'Modified' ? sort.direction : 'Hidden',
      },
    ],
    [isMobile, sort],
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
          webVault.repoFilesBrowsersSortBy(
            browserId,
            RepoFilesSortFieldArg.Name,
          );
          break;
        case 'size':
          webVault.repoFilesBrowsersSortBy(
            browserId,
            RepoFilesSortFieldArg.Size,
          );
          break;
        case 'modified':
          webVault.repoFilesBrowsersSortBy(
            browserId,
            RepoFilesSortFieldArg.Modified,
          );
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
  const onRowContextMenu = useCallback(
    (event: MouseEvent<HTMLElement>, index: number) => {},
    [],
  );

  return (
    <Table
      columns={columns}
      selectionSummary={info.selectionSummary}
      length={items.length}
      data={data}
      Row={RepoFilesTableRow}
      ariaLabel="Files list"
      onHeadCheckboxClick={onHeadCheckboxClick}
      onSortByClick={onSortByClick}
      onRowCheckboxClick={onRowCheckboxClick}
      onRowClick={onRowClick}
      onRowContextMenu={onRowContextMenu}
    />
  );
});
