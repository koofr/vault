import { ComponentType, memo, useState } from 'react';

import { lazyLoadingComponent } from '../lazyLoadingComponent';

import type { IntroModalProps } from './IntroModal';

const IntroModalLazyLoadingComponent = lazyLoadingComponent<IntroModalProps>(
  () => import('./IntroModal').then((mod) => mod.IntroModal),
  false
);

export const IntroModalLazy = memo<IntroModalProps>(({ isVisible, hide }) => {
  const [Component, setComponent] = useState<ComponentType<IntroModalProps>>();

  if (isVisible && Component === undefined) {
    setComponent(() => IntroModalLazyLoadingComponent);
  }

  return Component !== undefined ? (
    <Component isVisible={isVisible} hide={hide} />
  ) : null;
});
