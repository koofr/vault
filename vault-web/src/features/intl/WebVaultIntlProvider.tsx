import { memo, PropsWithChildren, useMemo } from 'react';
import { IntlProvider } from 'react-intl';

import { useSubscribe } from '../../webVault/useSubscribe';

import { WebVaultIntlLocalesProvider } from './WebVaultIntlLocalesProvider';
import { getMessages } from './getMessages';

export const WebVaultIntlProvider = memo<PropsWithChildren>(({ children }) => {
  const [locale] = useSubscribe(
    (v, cb) => v.intlCurrentLocaleSubscribe(cb),
    (v) => v.intlCurrentLocaleData,
    [],
  );

  const messages = useMemo(() => getMessages(locale?.locale), [locale]);

  if (locale === undefined) {
    return null;
  }

  return (
    <IntlProvider locale={locale.locale} messages={messages}>
      <WebVaultIntlLocalesProvider>{children}</WebVaultIntlLocalesProvider>
    </IntlProvider>
  );
});
WebVaultIntlProvider.displayName = 'WebVaultIntlProvider';
