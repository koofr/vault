import { memo, useMemo } from 'react';

import { NavbarBreadcrumbInfo } from '../../components/navbar/NavbarBreadcrumb';
import { NavbarBreadcrumbs } from '../../components/navbar/NavbarBreadcrumbs';
import { RemoteFilesBreadcrumb } from '../../vault-wasm/vault-wasm';

export const RemoteFilesBreadcrumbs = memo<{
  breadcrumbs: RemoteFilesBreadcrumb[];
}>(({ breadcrumbs }) => {
  const navbarBreadcrumbs = useMemo(
    () =>
      breadcrumbs.map((breadcrumb, i): NavbarBreadcrumbInfo => {
        return {
          id: breadcrumb.id,
          name: breadcrumb.name,
          isClickable: false,
          hasCaret: false,
          isLast: i === breadcrumbs.length - 1,
        };
      }),
    [breadcrumbs]
  );

  return <NavbarBreadcrumbs breadcrumbs={navbarBreadcrumbs} />;
});
