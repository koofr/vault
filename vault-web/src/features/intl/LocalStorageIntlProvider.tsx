import { memo, PropsWithChildren, useMemo, useState } from 'react';
import { IntlProvider } from 'react-intl';
import { match } from '@formatjs/intl-localematcher';

import {
  IntlLocales,
  IntlLocalesContext,
  IntlLocalesLocale,
} from './IntlLocalesContext';
import { getMessages } from './getMessages';
import localesJson from './locales/locales.json';

export const CURRENT_LOCALE_STORAGE_KEY = 'vaultIntlCurrentLocale';
export const DEFAULT_LOCALE = 'en';

const availableLocales = localesJson.map((l) => l.locale);

export const LocalStorageIntlProvider = memo<PropsWithChildren>(
  ({ children }) => {
    const locales: IntlLocalesLocale[] = localesJson;

    const [locale, setLocale] = useState<string>(() => {
      const value = localStorage.getItem(CURRENT_LOCALE_STORAGE_KEY);

      if (value === null) {
        return match(navigator.languages, availableLocales, DEFAULT_LOCALE);
      }

      return JSON.parse(value) as string;
    });

    const currentLocale = useMemo(() => {
      return locales.find((entry) => entry.locale === locale) ?? locales[0];
    }, [locale, locales]);

    const changeLocale = useMemo<IntlLocales['changeLocale']>(
      () => (locale: string) => {
        setLocale(locale);
        localStorage.setItem(
          CURRENT_LOCALE_STORAGE_KEY,
          JSON.stringify(locale),
        );
      },
      [],
    );

    const contextValue = useMemo(
      (): IntlLocales => ({
        currentLocale,
        locales,
        changeLocale,
      }),
      [changeLocale, currentLocale, locales],
    );

    const messages = useMemo(
      () => getMessages(currentLocale?.locale),
      [currentLocale],
    );

    return (
      <IntlProvider locale={currentLocale.locale} messages={messages}>
        <IntlLocalesContext.Provider value={contextValue}>
          {children}
        </IntlLocalesContext.Provider>
      </IntlProvider>
    );
  },
);
LocalStorageIntlProvider.displayName = 'LocalStorageIntlProvider';
