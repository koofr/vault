import type { Locale } from 'date-fns/locale';
import { sl } from 'date-fns/locale/sl';

export function getDateFnsLocale(locale: string): Locale | undefined {
  switch (locale) {
    case 'sl':
      return sl;
    default:
      return undefined;
  }
}
