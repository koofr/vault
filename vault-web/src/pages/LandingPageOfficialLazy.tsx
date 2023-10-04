import { lazyLoadingComponent } from '../components/lazyLoadingComponent';

export const LandingPageOfficialLazy = lazyLoadingComponent<{}>(
  () => import('./LandingPageOfficial').then((mod) => mod.LandingPageOfficial),
  false,
);
