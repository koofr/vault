import { memo } from 'react';

import { LandingPageOfficialLazy } from './LandingPageOfficialLazy';
import { LandingPageUnofficialLazy } from './LandingPageUnofficialLazy';

export const LandingPage = memo(() => {
  const hostname = document.location.hostname;

  if (
    hostname === 'vault.koofr.net' ||
    hostname === '127.0.0.1' ||
    hostname === 'localhost'
  ) {
    return <LandingPageOfficialLazy />;
  } else {
    return <LandingPageUnofficialLazy />;
  }
});
