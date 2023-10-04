import { memo, useMemo } from 'react';

import { FileIconProps } from '../../vault-wasm/vault-wasm';

import { createContext, PropsWithChildren, useContext, useRef } from 'react';
import { useWebVault } from '../../webVault/useWebVault';

export interface FileIconCache {
  getIcon(props: FileIconProps): string;
}

export const FileIconCacheContext = createContext<FileIconCache>(
  undefined as any,
);

export const FileIconCacheProvider: React.FC<PropsWithChildren<{}>> = ({
  children,
}) => {
  const webVault = useWebVault();
  const cache = useRef(new Map<string, string>());
  const api = useMemo((): FileIconCache => {
    return {
      getIcon(props) {
        const key = JSON.stringify(props);

        const value = cache.current.get(key);

        if (value !== undefined) {
          return value;
        }

        const svg = webVault.fileIconSvg(props);

        const url = `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;

        cache.current.set(key, url);

        return url;
      },
    };
  }, [webVault]);

  return (
    <FileIconCacheContext.Provider value={api}>
      {children}
    </FileIconCacheContext.Provider>
  );
};

export function useFileIconCache(): FileIconCache {
  return useContext(FileIconCacheContext);
}

export const FileIcon = memo<FileIconProps>((props) => {
  const cache = useFileIconCache();

  const url = cache.getIcon(props);

  return <img src={url} alt="File icon" />;
});
