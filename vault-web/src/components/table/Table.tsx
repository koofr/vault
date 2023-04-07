import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import range from 'lodash/range';
import {
  ComponentType,
  createContext,
  memo,
  MouseEvent,
  PropsWithChildren,
  useCallback,
  useContext,
  useMemo,
} from 'react';

import { withReactCss } from '../../styles';
import { useElementSize } from '../../utils/useElementSize';

import { Checkbox } from '../Checkbox';

import { calculateColumnWidths } from './columnWidths';

export type SortBy = 'Hidden' | 'Asc' | 'Desc';

export const TableCell = styled.div`
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  position: relative;
  align-items: center;
  padding-left: 10px;
  flex-grow: 0;
  flex-shrink: 0;
`;

export const TableHeadCell = styled(TableCell)`
  font-size: 13px;
  font-weight: normal;
  color: ${({ theme }) => theme.colors.textLight};
  vertical-align: middle;
`;

export const TableHeadCellCheckbox = styled(TableHeadCell)`
  align-items: center;
  justify-content: center;
  display: flex;
`;

export const TableSortLabel = styled.span<{ sortBy?: SortBy }>`
  color: ${({ theme }) => theme.colors.textLight};
  cursor: pointer;
  padding-right: 15px;

  ${({ sortBy, theme }) =>
    sortBy === 'Asc' || sortBy === 'Desc'
      ? withReactCss(
          (css) => css`
            &:after {
              content: '';
              display: inline-block;
              width: 0;
              height: 0;
              border-top: ${sortBy === 'Asc'
                ? 'none'
                : `4px solid ${theme.colors.textLight}`};
              border-bottom: ${sortBy === 'Asc'
                ? `4px solid ${theme.colors.textLight}`
                : 'none'};
              border-right: 4px solid transparent;
              border-left: 4px solid transparent;
              position: relative;
              left: 7px;
              bottom: 2px;
            }
          `
        )
      : undefined}
`;

export const TableHead = styled.div`
  display: flex;
  align-items: center;
  height: 33px;
  border-bottom: 1px solid ${({ theme }) => theme.colors.border};
  position: relative;
`;

export const TableBody = styled.div``;

export interface BaseTableRowProps {
  index: number;
  isSelected: boolean;
  isFirstSelected: boolean;
  hasHover: boolean;
  isDropOver: boolean;
  height: number;
  onClick: (event: MouseEvent<HTMLDivElement>) => void;
  onContextMenu: (event: MouseEvent<HTMLDivElement>) => void;
}

export const BaseTableRow = memo<PropsWithChildren<BaseTableRowProps>>(
  ({
    index,
    isSelected,
    isFirstSelected,
    hasHover,
    isDropOver,
    height,
    onClick,
    onContextMenu,
    children,
  }) => {
    const theme = useTheme();

    return (
      <div
        className={cx(
          css`
            display: flex;
            flex-direction: column;
            margin-top: -1px;
            background-color: #fff;
            border-top: 1px solid ${theme.colors.borderLight};
            border-bottom: 1px solid ${theme.colors.borderLight};
            border-left: 1px solid transparent;
            border-right: 1px solid transparent;
            overflow: hidden;
            position: relative;
            transition: height 0.3s ease-out, border 0.1s ease-out,
              box-shadow 0.1s ease-out;
          `,
          index === 0
            ? css`
                border-top-color: ${theme.colors.border};
              `
            : undefined,
          hasHover && !isSelected
            ? css`
                &:hover {
                  background-color: ${theme.colors.empty};
                  border-top-color: ${theme.colors.border};
                  border-left-color: ${theme.colors.border};
                  border-right-color: ${theme.colors.border};
                  border-bottom-color: ${theme.colors.border};
                  box-shadow: ${theme.boxShadow};
                  z-index: 1;
                }
              `
            : undefined,
          isSelected
            ? css`
                background-color: ${theme.colors.selectionBg};
                border-top-color: ${theme.colors.borderLight};
                border-left-color: ${theme.colors.selection};
                border-right-color: ${theme.colors.selection};
                border-bottom-color: ${theme.colors.selection};
                box-shadow: ${theme.boxShadow};
                z-index: 2;
              `
            : undefined,
          isFirstSelected
            ? css`
                border-top-color: ${theme.colors.selection};
              `
            : undefined,
          isDropOver
            ? css`
                background-color: ${theme.colors.bgLight};
                border-top-color: ${theme.colors.border};
                border-left-color: ${theme.colors.border};
                border-right-color: ${theme.colors.border};
                border-bottom-color: ${theme.colors.border};
                z-index: 1;
              `
            : undefined
        )}
        style={{ height: `${height + 1}px` }}
        onClick={onClick}
        onContextMenu={onContextMenu}
      >
        {children}
      </div>
    );
  }
);

export const TableCells = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  flex-shrink: 0;
  height: 47px;
`;

export const StyledTable = styled.div`
  user-select: none;
  padding: 0 2px 3px;
`;

export interface Column {
  name: string;
  label: string;
  width?: number | string;
  minWidth?: number;
  sortBy?: SortBy;
}

export interface ComputedColumn {
  name: string;
  label: string;
  width: number;
  sortBy?: SortBy;
}

export interface RowProps<T = any | undefined> {
  index: number;
  data: T;
}

export interface TableContextType {
  columns: ComputedColumn[];
  onRowCheckboxClick: (
    event: React.MouseEvent<HTMLElement>,
    index: number
  ) => void;
  onRowClick: (event: React.MouseEvent<HTMLElement>, index: number) => void;
  onRowContextMenu: (
    event: React.MouseEvent<HTMLElement>,
    index: number
  ) => void;
}

export const TableContext = createContext<TableContextType>(undefined as any);

export type TableSelectionSummary = 'None' | 'Partial' | 'All';

export interface TableProps<T = any> {
  columns: Column[];
  selectionSummary: TableSelectionSummary;
  length: number;
  data?: T;
  Row: ComponentType<RowProps<T>>;
  onHeadCheckboxClick: (event: MouseEvent) => void;
  onSortByClick?: (event: MouseEvent, columnName: string) => void;
  onRowCheckboxClick: (
    event: React.MouseEvent<HTMLElement>,
    index: number
  ) => void;
  onRowClick: (event: React.MouseEvent<HTMLElement>, index: number) => void;
  onRowContextMenu: (
    event: React.MouseEvent<HTMLElement>,
    index: number
  ) => void;
}

export const Table = memo<TableProps>(
  ({
    columns,
    selectionSummary,
    length,
    data,
    Row,
    onHeadCheckboxClick,
    onSortByClick,
    onRowCheckboxClick,
    onRowClick,
    onRowContextMenu,
  }) => {
    const [tableRef, { width: containerWidth }] = useElementSize();

    const computedColumns = useMemo(() => {
      if (containerWidth === 0) {
        return [];
      }

      const rowWidth = containerWidth - 6; // 2 * (1px border + 2px shadow)

      const allColumns: Column[] = [
        {
          name: 'checkbox',
          label: '',
          width: 42,
        },
        ...columns,
      ];

      const columnWidths = calculateColumnWidths(
        rowWidth,
        allColumns.map((column) => ({
          width: column.width,
          minWidth: column.minWidth,
        }))
      );

      return allColumns.map(
        (column, i): ComputedColumn => ({
          name: column.name,
          label: column.label,
          width: columnWidths[i],
          sortBy: column.sortBy,
        })
      );
    }, [columns, containerWidth]);

    const tableContext = useMemo(
      (): TableContextType => ({
        columns: computedColumns,
        onRowCheckboxClick,
        onRowClick,
        onRowContextMenu,
      }),
      [computedColumns, onRowCheckboxClick, onRowClick, onRowContextMenu]
    );

    const indexes = useMemo(() => range(0, length), [length]);

    return (
      <StyledTable ref={tableRef}>
        {computedColumns.length > 0 ? (
          <TableContext.Provider value={tableContext}>
            <TableHead>
              {computedColumns.map((column) =>
                column.name === 'checkbox' ? (
                  <TableHeadCellCheckbox
                    key={column.name}
                    style={{ minWidth: `${column.width}px` }}
                  >
                    <Checkbox
                      value={
                        selectionSummary === 'None'
                          ? 'unchecked'
                          : selectionSummary === 'Partial'
                          ? 'indeterminate'
                          : 'checked'
                      }
                      onClick={onHeadCheckboxClick}
                    />
                  </TableHeadCellCheckbox>
                ) : (
                  <TableHeadCell
                    key={column.name}
                    style={{ minWidth: `${column.width}px` }}
                  >
                    {column.sortBy !== undefined ? (
                      <TableSortLabel
                        sortBy={column.sortBy}
                        onClick={(event) => {
                          if (onSortByClick !== undefined) {
                            onSortByClick(event, column.name);
                          }
                        }}
                      >
                        {column.label}
                      </TableSortLabel>
                    ) : (
                      column.label
                    )}
                  </TableHeadCell>
                )
              )}
            </TableHead>
            <TableBody>
              {indexes.map((index) => (
                <Row key={index} index={index} data={data} />
              ))}
            </TableBody>
          </TableContext.Provider>
        ) : null}
      </StyledTable>
    );
  }
);

export interface TableRowProps {
  index: number;
  row: any;
  isSelected: boolean;
  isFirstSelected: boolean;
}

export const TableRow = memo<TableRowProps>(
  ({ index, row, isSelected, isFirstSelected }) => {
    const {
      columns,
      onRowCheckboxClick: onRowCheckboxClickOrig,
      onRowClick: onRowClickOrig,
      onRowContextMenu: onRowContextMenuOrig,
    } = useContext(TableContext);
    const onRowCheckboxClick = useCallback(
      (event: React.MouseEvent<HTMLElement>) => {
        onRowCheckboxClickOrig(event, index);
      },
      [onRowCheckboxClickOrig, index]
    );
    const onRowClick = useCallback(
      (event: React.MouseEvent<HTMLElement>) => {
        onRowClickOrig(event, index);
      },
      [onRowClickOrig, index]
    );
    const onRowContextMenu = useCallback(
      (event: React.MouseEvent<HTMLElement>) => {
        onRowContextMenuOrig(event, index);
      },
      [onRowContextMenuOrig, index]
    );

    return (
      <BaseTableRow
        key={index}
        index={index}
        isSelected={isSelected}
        isFirstSelected={isFirstSelected}
        hasHover={true}
        isDropOver={false}
        height={48}
        onClick={onRowClick}
        onContextMenu={onRowContextMenu}
      >
        <TableCells>
          {columns.map((column) =>
            column.name === 'checkbox' ? (
              <TableCell
                key={column.name}
                style={{ width: `${column.width}px` }}
              >
                <Checkbox
                  value={isSelected ? 'checked' : 'unchecked'}
                  onClick={onRowCheckboxClick}
                />
              </TableCell>
            ) : (
              <TableCell
                key={column.name}
                style={{ width: `${column.width}px` }}
              >
                {row[column.name]}
              </TableCell>
            )
          )}
        </TableCells>
      </BaseTableRow>
    );
  }
);
