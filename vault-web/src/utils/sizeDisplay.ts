const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'];

export interface HumanSize {
  size: string;
  unit: string;
}

export interface HumanSizeOf {
  used: HumanSize;
  total?: HumanSize;
}

function fixPoint(num: number): string {
  let fixed = num.toFixed(1);

  if (/\.0$/.test(fixed)) {
    fixed = fixed.substring(0, fixed.length - 2);
  }

  return fixed;
}

export function humanSize(bytes: number, baseUnit?: number): HumanSize {
  let unitPos = baseUnit || 0;

  while (bytes >= 1024) {
    unitPos += 1;
    bytes /= 1024;
  }

  return {
    size: fixPoint(bytes),
    unit: UNITS[unitPos],
  };
}

export function sizeDisplay(bytes: number, baseUnit?: number): string {
  const human = humanSize(bytes, baseUnit);

  return `${human.size} ${human.unit}`;
}

export function humanSizeOf(
  bytesUsed: number,
  bytesTotal: number | undefined,
  baseUnit?: number
): HumanSizeOf {
  if (bytesTotal === undefined) {
    return {
      used: humanSize(bytesUsed),
      total: undefined,
    };
  }

  let bytes = bytesTotal;

  let div = 1;
  let unitPos = baseUnit || 0;

  while (bytes >= 1024) {
    unitPos += 1;
    bytes /= 1024;
    div *= 1024;
  }

  return {
    used: {
      size: fixPoint(bytesUsed / div),
      unit: UNITS[unitPos],
    },
    total: {
      size: fixPoint(bytesTotal / div),
      unit: UNITS[unitPos],
    },
  };
}

export function sizeOfDisplay(
  bytesUsed: number,
  bytesTotal: number,
  baseUnit?: number
): string {
  const human = humanSizeOf(bytesUsed, bytesTotal, baseUnit);

  if (human.total === undefined) {
    return `${human.used.size} / ??? ${human.used.unit}`;
  }

  return `${human.used.size} / ${human.total.size} ${human.used.unit}`;
}
