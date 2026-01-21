import { createContext, useContext } from 'react';

export interface IntlLocalesLocale {
  locale: string;
  name: string;
}

export interface IntlLocales {
  currentLocale: IntlLocalesLocale;
  locales: IntlLocalesLocale[];
  changeLocale: (locale: string) => void;
}

export const IntlLocalesContext = createContext<IntlLocales | undefined>(
  undefined,
);

export const useIntlLocales = (): IntlLocales => {
  return useContext(IntlLocalesContext)!;
};
