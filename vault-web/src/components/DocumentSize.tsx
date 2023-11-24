import {
  createContext,
  PropsWithChildren,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react';

export interface DocumentSizeInfo {
  width: number;
  height: number;
}

export function getDocumentSize(): DocumentSizeInfo {
  const width = document.documentElement.clientWidth;
  const height = document.documentElement.clientHeight;

  return { width, height };
}

export const DocumentSizeContext = createContext<DocumentSizeInfo>(
  undefined as any,
);

export const DocumentSizeProvider: React.FC<PropsWithChildren<{}>> = ({
  children,
}) => {
  const [size, setSize] = useState<DocumentSizeInfo>(getDocumentSize);
  const lastSize = useRef(size);

  const onResize = useCallback(() => {
    const newSize = getDocumentSize();

    if (
      newSize.width !== lastSize.current.width ||
      newSize.height !== lastSize.current.height
    ) {
      setSize(newSize);
      lastSize.current = newSize;
    }
  }, []);

  onResize();

  useEffect(() => {
    window.addEventListener('resize', onResize);
    window.addEventListener('scroll', onResize);

    return () => {
      window.removeEventListener('resize', onResize);
      window.removeEventListener('scroll', onResize);
    };
  }, [onResize]);

  return (
    <DocumentSizeContext.Provider value={size}>
      {children}
    </DocumentSizeContext.Provider>
  );
};

export function useDocumentSize(): DocumentSizeInfo {
  const size = useContext(DocumentSizeContext);

  return size;
}
