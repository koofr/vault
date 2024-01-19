import { createContext } from 'react';

import { LandingPageLazy } from './pages/LandingPageLazy';

export const LandingPageContext = createContext<React.ReactNode>(
  <LandingPageLazy />,
);
