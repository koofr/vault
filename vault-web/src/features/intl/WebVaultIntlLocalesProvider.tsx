import { memo, PropsWithChildren, useMemo } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import {
  IntlLocales,
  IntlLocalesContext,
  IntlLocalesLocale,
} from './IntlLocalesContext';

export const WebVaultIntlLocalesProvider = memo<PropsWithChildren>(
  ({ children }) => {
    const webVault = useWebVault();
    const [currentLocale] = useSubscribe(
      (v, cb) => v.intlCurrentLocaleSubscribe(cb),
      (v) => v.intlCurrentLocaleData,
      [],
    );
    const [locales] = useSubscribe(
      (v, cb) => v.intlLocalesSubscribe(cb),
      (v) => v.intlLocalesData,
      [],
    );

    const contextValue = useMemo((): IntlLocales | undefined => {
      if (currentLocale === undefined || locales === undefined) {
        return undefined;
      }

      return {
        currentLocale: currentLocale as IntlLocalesLocale,
        locales: locales as IntlLocalesLocale[],
        changeLocale: (locale: string) => {
          webVault.intlChangeLocale(locale);
        },
      };
    }, [currentLocale, locales, webVault]);

    if (contextValue === undefined) {
      return null;
    }

    return (
      <IntlLocalesContext.Provider value={contextValue}>
        {children}
      </IntlLocalesContext.Provider>
    );
  },
);
WebVaultIntlLocalesProvider.displayName = 'WebVaultIntlLocalesProvider';
