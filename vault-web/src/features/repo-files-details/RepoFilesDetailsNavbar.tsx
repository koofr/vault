import { memo } from 'react';
import { To } from 'react-router-dom';

import { DashboardNavbar } from '../../components/dashboard/DashboardNavbar';
import { NavbarClose } from '../../components/navbar/NavbarClose';
import { RepoFilesDetailsInfo } from '../../vault-wasm/vault-wasm';

import { repoFilesDetailsLink, repoFilesLink } from '../repo-files/selectors';

import { RepoFilesDetailsNavbarHeader } from './RepoFilesDetailsNavbarHeader';
import { RepoFilesDetailsNavbarNav } from './RepoFilesDetailsNavbarNav';

const closeLink = (info: RepoFilesDetailsInfo): To => {
  if (
    info.isEditing &&
    info.repoId !== undefined &&
    info.encryptedPath !== undefined
  ) {
    return repoFilesDetailsLink(info.repoId, info.encryptedPath, false);
  } else if (info.repoId !== undefined) {
    return repoFilesLink(info.repoId, info.encryptedParentPath ?? '/');
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
