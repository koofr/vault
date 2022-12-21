import { ReactNode, useMemo } from 'react';

import { NavbarBreadcrumbInfo } from './NavbarBreadcrumb';
import { NavbarBreadcrumbs } from './NavbarBreadcrumbs';

export function useSingleNavbarBreadcrumb(name: string): ReactNode {
  const breadcrumbs = useMemo(
    (): NavbarBreadcrumbInfo[] => [
      {
        id: 'breadcrumb',
        name: name,
        isClickable: false,
        hasCaret: false,
        isLast: true,
      },
    ],
    [name]
  );

  return <NavbarBreadcrumbs breadcrumbs={breadcrumbs} />;
}
