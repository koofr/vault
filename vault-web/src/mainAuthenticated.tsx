import { CacheProvider } from '@emotion/react';
import { DndProvider } from 'react-dnd';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router-dom';
import streamSaver from 'streamsaver';

import { RemoveAppLoading } from './RemoveAppLoading';
import { DocumentScrollProvider } from './components/DocumentScroll';
import { DocumentSizeProvider } from './components/DocumentSize';
import { FolderAwareHTML5Backend } from './components/dnd/backend';
import { FileIconCacheProvider } from './components/file-icon/FileIcon';
import { ModalsProvider } from './components/modal/Modals';
import { Dialogs } from './features/dialogs/Dialogs';
import { Notifications } from './features/notifications/Notifications';
import { TransfersPreventUnload } from './features/transfers/TransfersPreventUnload';
import { createRouter } from './router';
import { GlobalStyles } from './styles/GlobalStyles';
import { emotionCache } from './styles/emotionCache';
import { DynamicThemeProvider } from './theme/DynamicThemeProvider';
import init, { WebVault, initConsole } from './vault-wasm/vault-wasm';
import { BrowserEventstreamWebSocketDelegateImpl } from './webVault/BrowserEventstreamWebSocketDelegateImpl';
import { BrowserHttpClientDelegateImpl } from './webVault/BrowserHttpClientDelegateImpl';
import { WebVaultContext } from './webVault/webVaultContext';

interface Config {
  baseUrl: string;
  oauth2ClientId: string;
  oauth2ClientSecret: string;
}

export const mainAuthenticated = async () => {
  const configPromise = fetch('/config.json').then(
    (res) => res.json() as Promise<Config>,
  );

  await init();
  initConsole();

  streamSaver.mitm =
    window.location.origin + '/streamsaver-2.0.6/mitm.html?version=2.0.0';

  const config = await configPromise;

  const baseUrl = config.baseUrl;
  const oauth2ClientId = config.oauth2ClientId;
  const oauth2ClientSecret = config.oauth2ClientSecret;
  const oauth2RedirectUri = window.location.origin + '/oauth2callback';

  const webVault = new WebVault(
    baseUrl,
    baseUrl,
    oauth2ClientId,
    oauth2ClientSecret,
    oauth2RedirectUri,
    new BrowserHttpClientDelegateImpl(),
    new BrowserEventstreamWebSocketDelegateImpl(),
    localStorage,
  );

  (window as any).webVault = webVault;

  // don't load the app on oauth2 login or logout
  if (document.location.pathname !== '/oauth2callback') {
    webVault.load();
  }

  const router = createRouter();

  (window as any).router = router;

  ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <WebVaultContext.Provider value={webVault}>
      <CacheProvider value={emotionCache}>
        <DocumentSizeProvider>
          <DocumentScrollProvider>
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
          </DocumentScrollProvider>
        </DocumentSizeProvider>
      </CacheProvider>
    </WebVaultContext.Provider>,
  );
};
