import { useDocumentSize } from './DocumentSize';

export function useIsMobile(): boolean {
  const documentSize = useDocumentSize();

  return documentSize.width < 768;
}
