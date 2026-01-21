import type { Locale } from 'date-fns/locale';
import { createContext, memo, PropsWithChildren, useContext } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';

import { getDateFnsLocale } from './getDateFnsLocale';

export const DateFnsLocaleContext = createContext<Locale | undefined>(
  undefined,
);

export const DateFnsLocaleProvider = memo<PropsWithChildren>(({ children }) => {
  const [locale] = useSubscribe(
    (v, cb) => v.intlCurrentLocaleSubscribe(cb),
    (v) => v.intlCurrentLocaleData,
    [],
  );

  const dateFnsLocale =
    locale !== undefined ? getDateFnsLocale(locale.locale) : undefined;

  return (
    <DateFnsLocaleContext.Provider value={dateFnsLocale}>
      {children}
    </DateFnsLocaleContext.Provider>
  );
});
DateFnsLocaleProvider.displayName = 'DateFnsLocaleProvider';

export const useDateFnsLocale = (): Locale | undefined => {
  return useContext(DateFnsLocaleContext);
};
