import { CacheProvider } from '@emotion/react';
import { Suspense } from 'react';
import { DndProvider } from 'react-dnd';
import { RouterProvider } from 'react-router-dom';

import { RemoveAppLoading } from './RemoveAppLoading';
import { DocumentScrollProvider } from './components/DocumentScroll';
import { DocumentSizeProvider } from './components/DocumentSize';
import { LoadingCircle } from './components/LoadingCircle';
import { FolderAwareHTML5Backend } from './components/dnd/backend';
import { FileIconCacheProvider } from './components/file-icon/FileIcon';
import { ModalsProvider } from './components/modal/Modals';
import { NavbarStickyProvider } from './components/navbar/NavbarSticky';
import { Config, ConfigContext } from './config';
import { Dialogs } from './features/dialogs/Dialogs';
import { Notifications } from './features/notifications/Notifications';
import { TransfersPreventUnload } from './features/transfers/TransfersPreventUnload';
import { LandingPageContext } from './landingPageContext';
import { createRouter } from './router';
import { GlobalStyles } from './styles/GlobalStyles';
import { emotionCache } from './styles/emotionCache';
import { DynamicThemeProvider } from './theme/DynamicThemeProvider';
import { WebVault } from './vault-wasm/vault-wasm';
import { WebVaultContext } from './webVault/webVaultContext';

export function getApp(
  config: Config,
  webVault: WebVault,
  landingPage: React.ReactNode,
): React.ReactNode {
  (window as any).webVault = webVault;

  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
      webVault.appVisible();
    } else {
      webVault.appHidden();
    }
  });

  const router = createRouter(landingPage);

  (window as any).router = router;

  return (
    <ConfigContext.Provider value={config}>
      <WebVaultContext.Provider value={webVault}>
        <LandingPageContext.Provider value={landingPage}>
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
                            <Suspense fallback={<LoadingCircle />}>
                              <RouterProvider router={router} />
                            </Suspense>

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
        </LandingPageContext.Provider>
      </WebVaultContext.Provider>
    </ConfigContext.Provider>
  );
}
