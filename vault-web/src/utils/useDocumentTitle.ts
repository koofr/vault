export function useDocumentTitle(title?: string) {
  const productName = 'Koofr Vault';

  document.title =
    title !== undefined ? `${title} - ${productName}` : productName;
}
