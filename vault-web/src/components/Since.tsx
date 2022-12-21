import format from 'date-fns/format';
import formatDistanceToNow from 'date-fns/formatDistanceToNow';
import { memo } from 'react';

export const Since = memo<{ value: number }>(({ value }) => {
  if (value > Date.now()) {
    value = Date.now();
  }

  return (
    <span title={format(value, 'PPPPpp')}>
      {formatDistanceToNow(value, { addSuffix: true })}
    </span>
  );
});
