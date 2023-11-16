import { memo } from 'react';

import { NavbarBack } from '../../components/navbar/NavbarBack';
import { NavbarLogo } from '../../components/navbar/NavbarLogo';
import { useIsMobile } from '../../components/useIsMobile';
import { RepoFilesBreadcrumb } from '../../vault-wasm/vault-wasm';

import { repoFilesLink } from './selectors';

export const RepoFilesNavbarLeft = memo<{
  breadcrumbs: RepoFilesBreadcrumb[];
}>(({ breadcrumbs }) => {
  const isMobile = useIsMobile();

  if (isMobile && breadcrumbs.length > 1) {
    const breadcrumb = breadcrumbs[breadcrumbs.length - 2];
    const link = repoFilesLink(breadcrumb.repoId, breadcrumb.path);

    return <NavbarBack to={link} />;
  } else {
    return <NavbarLogo />;
  }
});
