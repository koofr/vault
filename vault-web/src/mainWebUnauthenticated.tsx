import { CacheProvider } from '@emotion/react';
import ReactDOM from 'react-dom/client';

import { RemoveAppLoading } from './RemoveAppLoading';
import { DocumentScrollProvider } from './components/DocumentScroll';
import { DocumentSizeProvider } from './components/DocumentSize';
import { ConfigContext, loadConfig } from './config';
import { LandingPage } from './pages/LandingPage';
import { GlobalStyles } from './styles/GlobalStyles';
import { emotionCache } from './styles/emotionCache';
import { DynamicThemeProvider } from './theme/DynamicThemeProvider';

export const mainUnauthenticated = async (child = <LandingPage />) => {
  const config = await loadConfig();

  ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <ConfigContext.Provider value={config}>
      <CacheProvider value={emotionCache}>
        <DocumentSizeProvider>
          <DocumentScrollProvider>
            <DynamicThemeProvider>
              <GlobalStyles />

              <RemoveAppLoading />

              {child}
            </DynamicThemeProvider>
          </DocumentScrollProvider>
        </DocumentSizeProvider>
      </CacheProvider>
    </ConfigContext.Provider>,
  );
};
