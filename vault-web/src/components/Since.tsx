import { format } from 'date-fns/format';
import { memo, useEffect, useState } from 'react';

import { useDateFnsLocale } from '../features/intl/DateFnsLocaleContext';
import { useWebVault } from '../webVault/useWebVault';

export const Since = memo<{ value: number; noTooltip?: boolean }>(
  ({ value, noTooltip }) => {
    const webVault = useWebVault();
    const dateFnsLocale = useDateFnsLocale();

    const [relativeTime, setRelativeTime] = useState(() =>
      webVault.relativeTime(Math.min(value, Date.now()), true),
    );
    const nextUpdate = relativeTime.nextUpdate;

    useEffect(() => {
      if (nextUpdate !== undefined) {
        const timer = setTimeout(
          () => {
            setRelativeTime(
              webVault.relativeTime(Math.min(value, Date.now()), true),
            );
          },
          Math.max(nextUpdate - Date.now(), 0),
        );

        return () => {
          clearTimeout(timer);
        };
      }
    }, [webVault, value, nextUpdate]);

    return (
      <span
        title={
          noTooltip
            ? undefined
            : format(value, 'PPPPpp', {
                locale: dateFnsLocale,
              })
        }
      >
        {relativeTime.display}
      </span>
    );
  },
);
Since.displayName = 'Since';
