import {
  createContext,
  PropsWithChildren,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react';

export interface DocumentScrollInfo {
  y: number;
}

export function getDocumentScroll(): DocumentScrollInfo {
  const y = window.pageYOffset;

  return { y };
}

export const DocumentScrollContext = createContext<DocumentScrollInfo>(
  undefined as any,
);

export const DocumentScrollProvider: React.FC<PropsWithChildren<{}>> = ({
  children,
}) => {
  const [info, setInfo] = useState<DocumentScrollInfo>(getDocumentScroll);
  const lastInfo = useRef(info);

  const onScroll = useCallback(() => {
    const newInfo = getDocumentScroll();

    if (newInfo.y !== lastInfo.current.y) {
      setInfo(newInfo);
      lastInfo.current = newInfo;
    }
  }, []);

  onScroll();

  useEffect(() => {
    window.addEventListener('resize', onScroll);
    window.addEventListener('scroll', onScroll);

    return () => {
      window.removeEventListener('resize', onScroll);
      window.removeEventListener('scroll', onScroll);
    };
  });

  return (
    <DocumentScrollContext.Provider value={info}>
      {children}
    </DocumentScrollContext.Provider>
  );
};

export function useDocumentScroll(): DocumentScrollInfo {
  const size = useContext(DocumentScrollContext);

  return size;
}
