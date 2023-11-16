import { memo, useCallback, useMemo } from 'react';

import { NavbarBreadcrumbInfo } from '../../components/navbar/NavbarBreadcrumb';
import { NavbarBreadcrumbs } from '../../components/navbar/NavbarBreadcrumbs';
import { useIsMobile } from '../../components/useIsMobile';
import { RepoFilesBreadcrumb } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { repoFilesLink } from './selectors';

export const RepoFilesBreadcrumbs = memo<{
  breadcrumbs: RepoFilesBreadcrumb[];
}>(({ breadcrumbs }) => {
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();
  const visibleBreadcrumbs = useMemo(
    () => (isMobile ? breadcrumbs.slice(breadcrumbs.length - 1) : breadcrumbs),
    [breadcrumbs, isMobile],
  );
  const navbarBreadcrumbs = useMemo(
    () =>
      visibleBreadcrumbs.map((breadcrumb, i): NavbarBreadcrumbInfo => {
        return {
          id: breadcrumb.id,
          name: breadcrumb.name,
          link: repoFilesLink(breadcrumb.repoId, breadcrumb.path),
          isClickable: true,
          hasCaret: false,
          isLast: i === visibleBreadcrumbs.length - 1,
        };
      }),
    [visibleBreadcrumbs],
  );
  const onClick = useCallback(
    (event: React.MouseEvent, breadcrumb: NavbarBreadcrumbInfo) => {
      if (breadcrumb.isLast) {
        webVault.repoFilesBrowsersLoadFiles(browserId);
      }
    },
    [webVault, browserId],
  );

  return (
    <NavbarBreadcrumbs breadcrumbs={navbarBreadcrumbs} onClick={onClick} />
  );
});
