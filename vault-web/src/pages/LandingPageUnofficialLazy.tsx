import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const LandingPageUnofficialLazy = lazyLoadingComponent<{}>(
  () =>
    import('./LandingPageUnofficial').then((mod) => mod.LandingPageUnofficial),
  false
);
