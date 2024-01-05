import { CacheProvider } from '@emotion/react';
import { DndProvider } from 'react-dnd';
import { RouterProvider } from 'react-router-dom';

import { RemoveAppLoading } from './RemoveAppLoading';
import { DocumentScrollProvider } from './components/DocumentScroll';
import { DocumentSizeProvider } from './components/DocumentSize';
import { FolderAwareHTML5Backend } from './components/dnd/backend';
import { FileIconCacheProvider } from './components/file-icon/FileIcon';
import { ModalsProvider } from './components/modal/Modals';
import { NavbarStickyProvider } from './components/navbar/NavbarSticky';
import { Config, ConfigContext } from './config';
import { Dialogs } from './features/dialogs/Dialogs';
import { Notifications } from './features/notifications/Notifications';
import { TransfersPreventUnload } from './features/transfers/TransfersPreventUnload';
import { createRouter } from './router';
import { GlobalStyles } from './styles/GlobalStyles';
import { emotionCache } from './styles/emotionCache';
import { DynamicThemeProvider } from './theme/DynamicThemeProvider';
import { WebVaultContext } from './webVault/webVaultContext';
import { WebVault } from './vault-wasm/vault-wasm';

export function getApp(config: Config, webVault: WebVault): React.ReactNode {
  (window as any).webVault = webVault;

  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
      webVault.appVisible();
    } else {
      webVault.appHidden();
    }
  });

  const router = createRouter();

  (window as any).router = router;

  return (
    <ConfigContext.Provider value={config}>
      <WebVaultContext.Provider value={webVault}>
        <CacheProvider value={emotionCache}>
          <DocumentSizeProvider>
            <DocumentScrollProvider>
              <NavbarStickyProvider>
                <DynamicThemeProvider>
                  <DndProvider backend={FolderAwareHTML5Backend}>
                    <FileIconCacheProvider>
                      <>
                        <GlobalStyles />

                        <RemoveAppLoading />

                        <ModalsProvider>
                          <RouterProvider router={router} />

                          <Dialogs />
                        </ModalsProvider>

                        <Notifications />
                        <TransfersPreventUnload />
                      </>
                    </FileIconCacheProvider>
                  </DndProvider>
                </DynamicThemeProvider>
              </NavbarStickyProvider>
            </DocumentScrollProvider>
          </DocumentSizeProvider>
        </CacheProvider>
      </WebVaultContext.Provider>
    </ConfigContext.Provider>
  );
}
