import { DocumentSizeInfo, useDocumentSize } from './DocumentSize';

export function isDocumentSizeMobile(documentSize: DocumentSizeInfo): boolean {
  return documentSize.width < 768;
}

export function useIsMobile(): boolean {
  const documentSize = useDocumentSize();

  return isDocumentSizeMobile(documentSize);
}
