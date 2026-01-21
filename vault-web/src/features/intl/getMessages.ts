import { MessageFormatElement } from 'react-intl';

const modules = import.meta.glob('./locales/*/compiled.json', {
  eager: true,
  import: 'default',
});

const messagesMap: Record<
  string,
  Record<string, MessageFormatElement[]>
> = Object.fromEntries(
  Object.entries(modules).map(([path, messages]) => [
    // "./locales/en/compiled.json" -> "en"
    path.split('/')[2],
    messages as Record<string, MessageFormatElement[]>,
  ]),
);

export function getMessages(
  locale: string | undefined,
): Record<string, MessageFormatElement[]> {
  if (locale === undefined) {
    return {};
  }

  return messagesMap[locale] ?? {};
}
