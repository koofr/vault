import isString from 'lodash/isString';

export interface ColumnWidth {
  width?: number | string;
  minWidth?: number;
}

enum ColumnType {
  UNKNOWN,
  FIXED,
  RELATIVE,
}

interface Col {
  type: ColumnType;
  width?: number;
  relWidth?: number;
  minWidth?: number;
}

export function calculateColumnWidths(
  availableWidth: number,
  columns: ColumnWidth[],
): number[] {
  let knownWidth = 0;
  let unknownCount = 0;
  let relWidthsSum = 0;

  const cols: Col[] = columns.map((column, i) => {
    const col: Col = {
      type: ColumnType.UNKNOWN,
      width: undefined,
      relWidth: undefined,
      minWidth: column.minWidth,
    };

    if (column.width === undefined) {
      unknownCount += 1;
    } else {
      if (isString(column.width)) {
        if (column.width[column.width.length - 1] !== '%') {
          throw new Error(
            `Column width must be a number or percentage: ${column.width}`,
          );
        }

        const relWidth =
          parseInt(column.width.slice(0, column.width.length - 1), 10) / 100;

        relWidthsSum += relWidth;

        col.type = ColumnType.RELATIVE;
        col.relWidth = relWidth;
      } else {
        if (column.minWidth !== undefined) {
          throw new Error(
            `Column cannot have both width number and minWidth: ${column.width}, ${column.minWidth}`,
          );
        }

        col.type = ColumnType.FIXED;
        col.width = column.width;

        knownWidth += col.width;
      }
    }

    return col;
  });

  if (relWidthsSum > 1) {
    throw new Error('Columns relative widths cannot be more than 100%');
  }

  if (unknownCount > 0) {
    if (relWidthsSum === 1) {
      throw new Error(
        'Columns relative widths cannot equal 100% if there are unknown widths',
      );
    }

    const unknownColumnsRelWidth = (1 - relWidthsSum) / unknownCount;

    cols.forEach((col) => {
      if (col.type === ColumnType.UNKNOWN) {
        col.type = ColumnType.RELATIVE;
        col.relWidth = unknownColumnsRelWidth;
      }
    });
  } else {
    cols.forEach((col) => {
      if (col.type === ColumnType.RELATIVE) {
        col.relWidth = (col.relWidth as number) / relWidthsSum;
      }
    });
  }

  const remainingWidth = Math.max(availableWidth - knownWidth, 0);

  let currentRemainingWidth = remainingWidth;

  cols.forEach((col) => {
    if (col.type === ColumnType.RELATIVE && col.minWidth !== undefined) {
      const width = Math.min(
        Math.max(remainingWidth * (col.relWidth as number), col.minWidth),
        currentRemainingWidth,
      );

      currentRemainingWidth -= width;

      col.type = ColumnType.FIXED;
      col.width = width;
    }
  });

  cols.forEach((col) => {
    if (col.type === ColumnType.RELATIVE) {
      const width = Math.min(
        Math.max(remainingWidth * (col.relWidth as number), 0),
        currentRemainingWidth,
      );

      currentRemainingWidth -= width;

      col.type = ColumnType.FIXED;
      col.width = width;
    }
  });

  return cols.map((col) => col.width as number);
}
