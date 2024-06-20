import { lazy, memo } from 'react';

import type { IntroModalProps } from './IntroModal';

const IntroModalLazyLoadingComponent = lazy(() =>
  import('./IntroModal').then((mod) => ({ default: mod.IntroModal })),
);

export const IntroModalLazy = memo<IntroModalProps>(({ isVisible, hide }) => {
  return isVisible ? (
    <IntroModalLazyLoadingComponent isVisible={isVisible} hide={hide} />
  ) : null;
});
