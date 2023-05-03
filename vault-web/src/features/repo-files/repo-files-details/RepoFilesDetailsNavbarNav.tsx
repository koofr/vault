import { memo } from 'react';
import { Link } from 'react-router-dom';

import { ReactComponent as FilesEditHoverIcon } from '../../../assets/images/files-edit-hover.svg';
import { ReactComponent as FilesEditIcon } from '../../../assets/images/files-edit.svg';
import { ReactComponent as FilesRenameHoverIcon } from '../../../assets/images/files-rename-hover.svg';
import { ReactComponent as FilesRenameIcon } from '../../../assets/images/files-rename.svg';
import { ReactComponent as FilesToolbarDeleteHoverIcon } from '../../../assets/images/files-toolbar-delete-hover.svg';
import { ReactComponent as FilesToolbarDeleteIcon } from '../../../assets/images/files-toolbar-delete.svg';
import { ReactComponent as FilesToolbarDownloadHoverIcon } from '../../../assets/images/files-toolbar-download-hover.svg';
import { ReactComponent as FilesToolbarDownloadIcon } from '../../../assets/images/files-toolbar-download.svg';
import {
  NavbarNavToolbar,
  NavbarNavToolbarItem,
} from '../../../components/navbar/NavbarNavToolbar';
import { useIsMobile } from '../../../components/useIsMobile';
import { RepoFilesDetailsInfo } from '../../../vault-wasm/vault-wasm';
import { useWebVault } from '../../../webVault/useWebVault';

import { downloadFile } from '../repoFilesActions';
import { fileCategoryHasDetailsEdit, repoFilesDetailsLink } from '../selectors';

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
