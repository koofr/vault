import { CacheProvider } from '@emotion/react';
import ReactDOM from 'react-dom/client';

import { RemoveAppLoading } from './RemoveAppLoading';
import { DocumentScrollProvider } from './components/DocumentScroll';
import { DocumentSizeProvider } from './components/DocumentSize';
import { LandingPage } from './pages/LandingPage';
import { GlobalStyles } from './styles/GlobalStyles';
import { emotionCache } from './styles/emotionCache';
import { DynamicThemeProvider } from './theme/DynamicThemeProvider';

export const mainUnauthenticated = (child = <LandingPage />) => {
  ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
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
    </CacheProvider>,
  );
};
