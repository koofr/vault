import { ThemeProvider } from '@emotion/react';
import { memo, PropsWithChildren, useMemo } from 'react';

import { useIsMobile } from '../components/useIsMobile';

import * as theme from './theme';

export const DynamicThemeProvider = memo<PropsWithChildren>(({ children }) => {
  const isMobile = useIsMobile();
  const dynamicTheme = useMemo(() => ({ ...theme, isMobile }), [isMobile]);

  return <ThemeProvider theme={dynamicTheme}>{children}</ThemeProvider>;
});
