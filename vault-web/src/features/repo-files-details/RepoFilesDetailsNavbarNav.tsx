import { memo } from 'react';
import { Link } from 'react-router-dom';

import FilesEditHoverIcon from '../../assets/images/files-edit-hover.svg?react';
import FilesEditIcon from '../../assets/images/files-edit.svg?react';
import FilesRenameHoverIcon from '../../assets/images/files-rename-hover.svg?react';
import FilesRenameIcon from '../../assets/images/files-rename.svg?react';
import FilesToolbarDeleteHoverIcon from '../../assets/images/files-toolbar-delete-hover.svg?react';
import FilesToolbarDeleteIcon from '../../assets/images/files-toolbar-delete.svg?react';
import FilesToolbarDownloadHoverIcon from '../../assets/images/files-toolbar-download-hover.svg?react';
import FilesToolbarDownloadIcon from '../../assets/images/files-toolbar-download.svg?react';
import {
  NavbarNavToolbar,
  NavbarNavToolbarItem,
} from '../../components/navbar/NavbarNavToolbar';
import { useIsMobile } from '../../components/useIsMobile';
import { RepoFilesDetailsInfo } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { downloadFile } from '../repo-files/repoFilesActions';
import {
  fileCategoryHasDetailsEdit,
  repoFilesDetailsLink,
} from '../repo-files/selectors';

export const RepoFilesDetailsNavbarNav = memo<{
  detailsId: number;
  info: RepoFilesDetailsInfo;
}>(({ detailsId, info }) => {
  const isMobile = useIsMobile();
  const webVault = useWebVault();

  return (
    <NavbarNavToolbar>
      {info.isEditing ? (
        <>
          <NavbarNavToolbarItem
            icon={<FilesRenameIcon role="img" />}
            iconHover={<FilesRenameHoverIcon role="img" />}
            onClick={() => {
              webVault.repoFilesDetailsSave(detailsId);
            }}
            disabled={!info.canSave}
          >
            Save
          </NavbarNavToolbarItem>
        </>
      ) : info.fileExists ? (
        <>
          {fileCategoryHasDetailsEdit(info.fileCategory) &&
          info.repoId !== undefined &&
          info.path !== undefined ? (
            <NavbarNavToolbarItem
              as={Link}
              to={repoFilesDetailsLink(info.repoId, info.path, true)}
              icon={<FilesEditIcon role="img" />}
              iconHover={<FilesEditHoverIcon role="img" />}
            >
              Edit
            </NavbarNavToolbarItem>
          ) : null}
          <NavbarNavToolbarItem
            icon={<FilesToolbarDownloadIcon role="img" />}
            iconHover={<FilesToolbarDownloadHoverIcon role="img" />}
            onClick={() => {
              if (info.repoId !== undefined && info.path !== undefined) {
                downloadFile(webVault, info.repoId, info.path, isMobile);
              }
            }}
          >
            Download
          </NavbarNavToolbarItem>
          <NavbarNavToolbarItem
            icon={<FilesRenameIcon role="img" />}
            iconHover={<FilesRenameHoverIcon role="img" />}
            onClick={() => {
              if (info.repoId !== undefined && info.path !== undefined) {
                webVault.repoFilesRenameFile(info.repoId, info.path);
              }
            }}
          >
            Rename
          </NavbarNavToolbarItem>
          <NavbarNavToolbarItem
            icon={<FilesToolbarDeleteIcon role="img" />}
            iconHover={<FilesToolbarDeleteHoverIcon role="img" />}
            onClick={() => {
              webVault.repoFilesDetailsDelete(detailsId);
            }}
          >
            Delete
          </NavbarNavToolbarItem>
        </>
      ) : null}
    </NavbarNavToolbar>
  );
});
