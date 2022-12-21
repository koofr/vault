import { memo } from 'react';

import { NavbarBack } from '../../components/navbar/NavbarBack';
import { NavbarLogo } from '../../components/navbar/NavbarLogo';
import { useIsMobile } from '../../components/useIsMobile';
import { useSubscribe } from '../../webVault/useSubscribe';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { repoFilesLink } from './repoFilesLink';

export const RepoFilesNavbarLeft = memo(() => {
  const isMobile = useIsMobile();
  const browserId = useRepoFilesBrowserId();
  const breadcrumbs = useSubscribe(
    (v, cb) => v.repoFilesBrowsersBreadcrumbsSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersBreadcrumbsData,
    [browserId]
  );

  if (isMobile && breadcrumbs.length > 1) {
    const breadcrumb = breadcrumbs[breadcrumbs.length - 2];
    const link = repoFilesLink(breadcrumb.repoId, breadcrumb.path);

    return <NavbarBack to={link} />;
  } else {
    return <NavbarLogo />;
  }
});
