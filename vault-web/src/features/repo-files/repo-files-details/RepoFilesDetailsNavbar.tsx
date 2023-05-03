import { memo } from 'react';
import { To } from 'react-router-dom';

import { DashboardNavbar } from '../../../components/dashboard/DashboardNavbar';
import { NavbarClose } from '../../../components/navbar/NavbarClose';
import { RepoFilesDetailsInfo } from '../../../vault-wasm/vault-wasm';

import { repoFilesDetailsLink, repoFilesLink } from '../selectors';

import { RepoFilesDetailsNavbarHeader } from './RepoFilesDetailsNavbarHeader';
import { RepoFilesDetailsNavbarNav } from './RepoFilesDetailsNavbarNav';

const closeLink = (info: RepoFilesDetailsInfo): To => {
  if (info.isEditing && info.repoId !== undefined && info.path !== undefined) {
    return repoFilesDetailsLink(info.repoId, info.path, false);
  } else if (info.repoId !== undefined) {
    return repoFilesLink(info.repoId, info.parentPath ?? '/');
  } else {
    return '/';
  }
};

export const RepoFilesDetailsNavbar = memo<{
  detailsId: number;
  info: RepoFilesDetailsInfo;
}>(({ detailsId, info }) => {
  return (
    <DashboardNavbar
      header={<RepoFilesDetailsNavbarHeader info={info} />}
      right={<NavbarClose to={closeLink(info)} />}
      nav={<RepoFilesDetailsNavbarNav detailsId={detailsId} info={info} />}
      noShadow
    />
  );
});
