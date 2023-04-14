import { memo, useCallback, useMemo } from 'react';

import { NavbarBreadcrumbInfo } from '../../components/navbar/NavbarBreadcrumb';
import { NavbarBreadcrumbs } from '../../components/navbar/NavbarBreadcrumbs';
import { useIsMobile } from '../../components/useIsMobile';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { repoFilesLink } from './selectors';

export const RepoFilesBreadcrumbs = memo(() => {
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();
  const allBreadcrumbs = useSubscribe(
    (v, cb) => v.repoFilesBrowsersBreadcrumbsSubscribe(browserId, cb),
    (v) => v.repoFilesBrowsersBreadcrumbsData,
    [browserId]
  );
  const breadcrumbs = useMemo(
    () =>
      isMobile
        ? allBreadcrumbs.slice(allBreadcrumbs.length - 1)
        : allBreadcrumbs,
    [allBreadcrumbs, isMobile]
  );
  const navbarBreadcrumbs = useMemo(
    () =>
      breadcrumbs.map((breadcrumb, i): NavbarBreadcrumbInfo => {
        return {
          id: breadcrumb.id,
          name: breadcrumb.name,
          link: repoFilesLink(breadcrumb.repoId, breadcrumb.path),
          isClickable: true,
          hasCaret: false,
          isLast: i === breadcrumbs.length - 1,
        };
      }),
    [breadcrumbs]
  );
  const onClick = useCallback(
    (event: React.MouseEvent<any>, breadcrumb: NavbarBreadcrumbInfo) => {
      if (breadcrumb.isLast) {
        webVault.repoFilesBrowsersLoadFiles(browserId);
      }
    },
    [webVault, browserId]
  );

  return (
    <NavbarBreadcrumbs breadcrumbs={navbarBreadcrumbs} onClick={onClick} />
  );
});
